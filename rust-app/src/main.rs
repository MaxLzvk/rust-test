use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct Command {
    id: i32,
    command: String,
    definition: String,
    category: String,
}

#[derive(Serialize)]
struct MazeItem {
    label: String,
    kind: String,
    pair_id: i32,
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct ValidateRequest {
    command: String,
    definition: String,
}

#[derive(Serialize)]
struct ValidateResponse {
    correct: bool,
    command: String,
    definition: String,
}

#[derive(Serialize)]
struct ScoreResponse {
    score: i32,
    total: i32,
}

struct AppState {
    db_path: String,
    score: Mutex<i32>,
    total: Mutex<i32>,
}

async fn get_commands(data: web::Data<AppState>) -> impl Responder {
    let conn = Connection::open(&data.db_path).unwrap();
    let mut stmt = conn
        .prepare("SELECT id, command, definition, category FROM commands")
        .unwrap();
    let commands: Vec<Command> = stmt
        .query_map([], |row| {
            Ok(Command {
                id: row.get(0)?,
                command: row.get(1)?,
                definition: row.get(2)?,
                category: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|c| c.ok())
        .collect();
    HttpResponse::Ok().json(commands)
}

async fn get_maze(data: web::Data<AppState>) -> impl Responder {
    let conn = Connection::open(&data.db_path).unwrap();
    let mut stmt = conn
        .prepare("SELECT id, command, definition FROM commands ORDER BY RANDOM() LIMIT 6")
        .unwrap();
    let pairs: Vec<(i32, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut rng = thread_rng();
    let mut maze_items: Vec<MazeItem> = Vec::new();

    for (id, cmd, def) in &pairs {
        maze_items.push(MazeItem {
            label: cmd.clone(),
            kind: "command".to_string(),
            pair_id: *id,
            x: 0,
            y: 0,
        });
        maze_items.push(MazeItem {
            label: def.clone(),
            kind: "definition".to_string(),
            pair_id: *id,
            x: 0,
            y: 0,
        });
    }

    maze_items.shuffle(&mut rng);

    let positions: Vec<(i32, i32)> = vec![
        (3, 1), (7, 1), (11, 1),
        (1, 3), (5, 3), (9, 3), (13, 3),
        (3, 5), (7, 5), (11, 5),
        (1, 7), (5, 7),
    ];

    for (i, item) in maze_items.iter_mut().enumerate() {
        if i < positions.len() {
            item.x = positions[i].0;
            item.y = positions[i].1;
        }
    }

    *data.total.lock().unwrap() = pairs.len() as i32;
    HttpResponse::Ok().json(maze_items)
}

async fn validate(
    req: web::Json<ValidateRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let conn = Connection::open(&data.db_path).unwrap();
    let mut stmt = conn
        .prepare("SELECT definition FROM commands WHERE command = ?1")
        .unwrap();
    let result: Result<String, _> = stmt.query_row(params![&req.command], |row| row.get(0));

    let correct = match result {
        Ok(def) => def == req.definition,
        Err(_) => false,
    };

    if correct {
        *data.score.lock().unwrap() += 1;
    }

    HttpResponse::Ok().json(ValidateResponse {
        correct,
        command: req.command.clone(),
        definition: req.definition.clone(),
    })
}

async fn get_score(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(ScoreResponse {
        score: *data.score.lock().unwrap(),
        total: *data.total.lock().unwrap(),
    })
}

async fn reset_score(data: web::Data<AppState>) -> impl Responder {
    *data.score.lock().unwrap() = 0;
    HttpResponse::Ok().json(ScoreResponse {
        score: 0,
        total: *data.total.lock().unwrap(),
    })
}

fn init_db(db_path: &str) {
    std::fs::create_dir_all("./data").unwrap();
    let mut conn = Connection::open(db_path).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS commands (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            definition TEXT NOT NULL,
            category TEXT
        )",
        [],
    )
    .unwrap();

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
        .unwrap();

    if count == 0 {
        let commands = vec![
            // === FICHIERS & NAVIGATION ===
            ("ls", "Lister les fichiers et dossiers", "fichiers"),
            ("cd", "Changer de repertoire", "fichiers"),
            ("pwd", "Afficher le repertoire courant", "fichiers"),
            ("cp", "Copier des fichiers ou dossiers", "fichiers"),
            ("mv", "Deplacer ou renommer des fichiers", "fichiers"),
            ("rm", "Supprimer des fichiers ou dossiers", "fichiers"),
            ("mkdir", "Creer un repertoire", "fichiers"),
            ("rmdir", "Supprimer un repertoire vide", "fichiers"),
            ("touch", "Creer un fichier vide", "fichiers"),
            ("cat", "Afficher le contenu d'un fichier", "fichiers"),
            ("less", "Lire un fichier page par page", "fichiers"),
            ("head", "Afficher les premieres lignes", "fichiers"),
            ("tail", "Afficher les dernieres lignes", "fichiers"),
            ("ln", "Creer un lien symbolique ou physique", "fichiers"),
            ("file", "Determiner le type d'un fichier", "fichiers"),
            ("stat", "Afficher les infos detaillees d'un fichier", "fichiers"),
            ("tree", "Afficher l'arborescence des dossiers", "fichiers"),
            ("basename", "Extraire le nom de fichier d'un chemin", "fichiers"),
            ("dirname", "Extraire le repertoire d'un chemin", "fichiers"),
            ("realpath", "Afficher le chemin absolu d'un fichier", "fichiers"),

            // === RECHERCHE & TEXTE ===
            ("grep", "Rechercher du texte dans des fichiers", "recherche"),
            ("find", "Rechercher des fichiers par criteres", "recherche"),
            ("locate", "Trouver un fichier rapidement via index", "recherche"),
            ("which", "Trouver l'emplacement d'une commande", "recherche"),
            ("whereis", "Localiser binaire, source et man d'une commande", "recherche"),
            ("wc", "Compter les lignes, mots et caracteres", "texte"),
            ("sort", "Trier les lignes d'un fichier", "texte"),
            ("uniq", "Supprimer les lignes en double", "texte"),
            ("cut", "Extraire des colonnes d'un fichier", "texte"),
            ("tr", "Remplacer ou supprimer des caracteres", "texte"),
            ("sed", "Editeur de flux pour transformer du texte", "texte"),
            ("awk", "Langage de traitement de texte ligne par ligne", "texte"),
            ("diff", "Comparer deux fichiers ligne par ligne", "texte"),
            ("tee", "Lire stdin et ecrire dans un fichier et stdout", "texte"),
            ("xargs", "Construire des commandes a partir de stdin", "texte"),

            // === ARCHIVAGE & COMPRESSION ===
            ("tar", "Archiver et compresser des fichiers", "archivage"),
            ("gzip", "Compresser un fichier en .gz", "archivage"),
            ("gunzip", "Decompresser un fichier .gz", "archivage"),
            ("zip", "Creer une archive .zip", "archivage"),
            ("unzip", "Extraire une archive .zip", "archivage"),

            // === PERMISSIONS & UTILISATEURS ===
            ("chmod", "Modifier les permissions d'un fichier", "permissions"),
            ("chown", "Changer le proprietaire d'un fichier", "permissions"),
            ("chgrp", "Changer le groupe d'un fichier", "permissions"),
            ("umask", "Definir les permissions par defaut", "permissions"),
            ("id", "Afficher l'UID, GID et groupes de l'utilisateur", "utilisateurs"),
            ("whoami", "Afficher le nom de l'utilisateur courant", "utilisateurs"),
            ("who", "Afficher les utilisateurs connectes", "utilisateurs"),
            ("w", "Afficher les utilisateurs connectes et leur activite", "utilisateurs"),
            ("useradd", "Creer un nouvel utilisateur", "utilisateurs"),
            ("userdel", "Supprimer un utilisateur", "utilisateurs"),
            ("usermod", "Modifier un compte utilisateur", "utilisateurs"),
            ("passwd", "Changer le mot de passe d'un utilisateur", "utilisateurs"),
            ("groupadd", "Creer un nouveau groupe", "utilisateurs"),
            ("groups", "Afficher les groupes d'un utilisateur", "utilisateurs"),
            ("su", "Changer d'utilisateur", "utilisateurs"),
            ("sudo", "Executer en superutilisateur", "utilisateurs"),

            // === PROCESSUS ===
            ("ps", "Afficher les processus en cours", "processus"),
            ("top", "Moniteur de processus temps reel", "processus"),
            ("htop", "Moniteur de processus interactif", "processus"),
            ("kill", "Envoyer un signal a un processus", "processus"),
            ("killall", "Tuer tous les processus par nom", "processus"),
            ("pkill", "Tuer des processus par motif de nom", "processus"),
            ("bg", "Reprendre un processus en arriere-plan", "processus"),
            ("fg", "Reprendre un processus en premier plan", "processus"),
            ("jobs", "Lister les processus en arriere-plan", "processus"),
            ("nohup", "Lancer un processus insensible au logout", "processus"),
            ("nice", "Lancer un processus avec une priorite modifiee", "processus"),
            ("crontab", "Planifier des taches automatiques", "processus"),

            // === SYSTEME ===
            ("uname", "Afficher les infos du systeme", "systeme"),
            ("hostname", "Afficher ou changer le nom de la machine", "systeme"),
            ("uptime", "Afficher le temps de fonctionnement", "systeme"),
            ("date", "Afficher ou modifier la date et l'heure", "systeme"),
            ("cal", "Afficher un calendrier", "systeme"),
            ("df", "Afficher l'espace disque disponible", "systeme"),
            ("du", "Afficher l'espace utilise par des fichiers", "systeme"),
            ("free", "Afficher la memoire disponible", "systeme"),
            ("lsblk", "Lister les peripheriques de stockage", "systeme"),
            ("mount", "Monter un systeme de fichiers", "systeme"),
            ("umount", "Demonter un systeme de fichiers", "systeme"),
            ("dmesg", "Afficher les messages du noyau", "systeme"),
            ("lsof", "Lister les fichiers ouverts", "systeme"),
            ("apt", "Gestionnaire de paquets Debian", "systeme"),
            ("pacman", "Gestionnaire de paquets Arch Linux", "systeme"),
            ("systemctl", "Gerer les services systemd", "systeme"),
            ("journalctl", "Consulter les logs systemd", "systeme"),
            ("shutdown", "Eteindre ou redemarrer la machine", "systeme"),
            ("reboot", "Redemarrer la machine", "systeme"),
            ("history", "Afficher l'historique des commandes", "systeme"),
            ("alias", "Creer un raccourci de commande", "systeme"),
            ("export", "Definir une variable d'environnement", "systeme"),
            ("env", "Afficher les variables d'environnement", "systeme"),
            ("echo", "Afficher du texte dans le terminal", "systeme"),
            ("man", "Afficher le manuel d'une commande", "systeme"),

            // === RESEAU ===
            ("ping", "Tester la connectivite reseau", "reseau"),
            ("ip", "Configurer les interfaces reseau", "reseau"),
            ("ifconfig", "Afficher la config reseau (ancien)", "reseau"),
            ("ss", "Afficher les connexions reseau", "reseau"),
            ("netstat", "Statistiques reseau (ancien)", "reseau"),
            ("curl", "Transferer des donnees via URL", "reseau"),
            ("wget", "Telecharger un fichier depuis le web", "reseau"),
            ("scp", "Copier des fichiers via SSH", "reseau"),
            ("ssh", "Se connecter a une machine distante", "reseau"),
            ("rsync", "Synchroniser des fichiers a distance", "reseau"),
            ("nslookup", "Interroger un serveur DNS", "reseau"),
            ("dig", "Requete DNS detaillee", "reseau"),
            ("traceroute", "Tracer le chemin reseau vers un hote", "reseau"),
            ("nmap", "Scanner les ports d'une machine", "reseau"),
            ("iptables", "Configurer le pare-feu Linux", "reseau"),
            ("ufw", "Pare-feu simplifie pour Ubuntu", "reseau"),

            // === DOCKER ===
            ("docker build", "Construire une image Docker", "docker"),
            ("docker run", "Lancer un conteneur Docker", "docker"),
            ("docker ps", "Lister les conteneurs en cours", "docker"),
            ("docker stop", "Arreter un conteneur", "docker"),
            ("docker rm", "Supprimer un conteneur", "docker"),
            ("docker images", "Lister les images Docker", "docker"),
            ("docker logs", "Afficher les logs d'un conteneur", "docker"),
            ("docker exec", "Executer une commande dans un conteneur", "docker"),
            ("docker compose up", "Demarrer les services Docker Compose", "docker"),
            ("docker compose down", "Arreter les services Docker Compose", "docker"),

            // === GIT ===
            ("git init", "Initialiser un depot Git", "git"),
            ("git clone", "Cloner un depot distant", "git"),
            ("git add", "Ajouter des fichiers au staging", "git"),
            ("git commit", "Enregistrer les modifications", "git"),
            ("git push", "Envoyer les commits vers le depot distant", "git"),
            ("git pull", "Recuperer et fusionner les modifications distantes", "git"),
            ("git status", "Afficher l'etat du depot", "git"),
            ("git log", "Afficher l'historique des commits", "git"),
            ("git branch", "Gerer les branches", "git"),
            ("git merge", "Fusionner une branche", "git"),
            ("git diff", "Afficher les differences entre fichiers", "git"),
        ];

        let tx = conn.transaction().unwrap();
        {
            let mut stmt = tx
                .prepare("INSERT INTO commands (command, definition, category) VALUES (?1, ?2, ?3)")
                .unwrap();
            for (cmd, def, cat) in &commands {
                stmt.execute(params![cmd, def, cat]).unwrap();
            }
        }
        tx.commit().unwrap();
        println!("DB initialisee avec {} commandes.", commands.len());
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = "./data/commands.db".to_string();
    init_db(&db_path);

    let app_state = web::Data::new(AppState {
        db_path,
        score: Mutex::new(0),
        total: Mutex::new(6),
    });

    println!("=== Maze Runner ===");
    println!("Listening on :8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/commands", web::get().to(get_commands))
            .route("/api/maze", web::get().to(get_maze))
            .route("/api/validate", web::post().to(validate))
            .route("/api/score", web::get().to(get_score))
            .route("/api/score/reset", web::post().to(reset_score))
            .service(Files::new("/", "./static/").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
