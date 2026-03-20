package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"
)

// Pokemon represents a caught Pokemon
type Pokemon struct {
	ID     int    `json:"id"`
	Name   string `json:"name"`
	Sprite string `json:"sprite"`
	Types  []string `json:"types"`
	HP     int    `json:"hp"`
	Attack int    `json:"attack"`
}

// TradeRequest represents an incoming trade
type TradeRequest struct {
	Pokemon  Pokemon `json:"pokemon"`
	FromIP   string  `json:"from_ip"`
	FromName string  `json:"from_name"`
}

// TradeOffer represents an outgoing trade offer
type TradeOffer struct {
	TargetIP  string `json:"target_ip"`
	PokemonID int    `json:"pokemon_id"`
}

var (
	collection []Pokemon
	mu         sync.Mutex
	trainerName string
)

func main() {
	name := flag.String("name", "Ash", "Nom du dresseur")
	port := flag.String("port", "3000", "Port d'ecoute")
	flag.Parse()

	trainerName = *name

	mux := http.NewServeMux()

	// Serve static files
	mux.Handle("/", http.FileServer(http.Dir("static")))

	// API routes
	mux.HandleFunc("/api/catch", handleCatch)
	mux.HandleFunc("/api/collection", handleCollection)
	mux.HandleFunc("/api/delete", handleDelete)
	mux.HandleFunc("/api/trade/send", handleTradeSend)
	mux.HandleFunc("/api/trade/receive", handleTradeReceive)
	mux.HandleFunc("/api/info", handleInfo)

	fmt.Printf("=== Pokemon Trader ===\n")
	fmt.Printf("Trainer: %s\n", trainerName)
	fmt.Printf("Listening on :%s\n", *port)
	fmt.Printf("Open http://localhost:%s in your browser\n\n", *port)

	if err := http.ListenAndServe(":"+*port, mux); err != nil {
		fmt.Printf("Error: %v\n", err)
	}
}

// handleCatch catches a random Pokemon from PokeAPI
func handleCatch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "POST only", http.StatusMethodNotAllowed)
		return
	}

	// Random Pokemon ID (gen 1-3, ids 1-386)
	id := rand.Intn(386) + 1

	pokemon, err := fetchPokemon(id)
	if err != nil {
		http.Error(w, "Failed to catch Pokemon: "+err.Error(), http.StatusInternalServerError)
		return
	}

	mu.Lock()
	collection = append(collection, pokemon)
	mu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(pokemon)
}

// handleCollection returns the current collection
func handleCollection(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	c := make([]Pokemon, len(collection))
	copy(c, collection)
	mu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(c)
}

// handleDelete removes a Pokemon from the collection
func handleDelete(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "POST only", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		PokemonID int `json:"pokemon_id"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request", http.StatusBadRequest)
		return
	}

	mu.Lock()
	var found bool
	for i, p := range collection {
		if p.ID == req.PokemonID {
			fmt.Printf("Released %s (#%d)\n", p.Name, p.ID)
			collection = append(collection[:i], collection[i+1:]...)
			found = true
			break
		}
	}
	mu.Unlock()

	if !found {
		http.Error(w, "Pokemon not in collection", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "released"})
}

// handleInfo returns trainer info
func handleInfo(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	count := len(collection)
	mu.Unlock()

	info := map[string]interface{}{
		"trainer":      trainerName,
		"pokemon_count": count,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(info)
}

// handleTradeSend sends a Pokemon to another trainer
func handleTradeSend(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "POST only", http.StatusMethodNotAllowed)
		return
	}

	var offer TradeOffer
	if err := json.NewDecoder(r.Body).Decode(&offer); err != nil {
		http.Error(w, "Invalid request", http.StatusBadRequest)
		return
	}

	// Find and remove the Pokemon from our collection
	mu.Lock()
	var found *Pokemon
	var idx int
	for i, p := range collection {
		if p.ID == offer.PokemonID {
			found = &collection[i]
			idx = i
			break
		}
	}

	if found == nil {
		mu.Unlock()
		http.Error(w, "Pokemon not in collection", http.StatusNotFound)
		return
	}

	pokemonToSend := *found
	collection = append(collection[:idx], collection[idx+1:]...)
	mu.Unlock()

	// Send to the other trainer
	trade := TradeRequest{
		Pokemon:  pokemonToSend,
		FromIP:   r.Host,
		FromName: trainerName,
	}

	body, _ := json.Marshal(trade)

	targetURL := fmt.Sprintf("http://%s/api/trade/receive", normalizeTarget(offer.TargetIP))
	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Post(targetURL, "application/json", strings.NewReader(string(body)))
	if err != nil {
		// Trade failed, give the Pokemon back
		mu.Lock()
		collection = append(collection, pokemonToSend)
		mu.Unlock()
		http.Error(w, "Trade failed: "+err.Error(), http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		mu.Lock()
		collection = append(collection, pokemonToSend)
		mu.Unlock()
		http.Error(w, "Trade rejected by target", http.StatusBadRequest)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status":  "traded",
		"pokemon": pokemonToSend.Name,
		"to":      offer.TargetIP,
	})
}

// handleTradeReceive accepts a Pokemon from another trainer
func handleTradeReceive(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "POST only", http.StatusMethodNotAllowed)
		return
	}

	var trade TradeRequest
	if err := json.NewDecoder(r.Body).Decode(&trade); err != nil {
		http.Error(w, "Invalid trade", http.StatusBadRequest)
		return
	}

	mu.Lock()
	collection = append(collection, trade.Pokemon)
	mu.Unlock()

	fmt.Printf("Received %s from %s (%s)!\n", trade.Pokemon.Name, trade.FromName, trade.FromIP)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "accepted"})
}

// fetchPokemon gets Pokemon data from PokeAPI
func fetchPokemon(id int) (Pokemon, error) {
	url := "https://pokeapi.co/api/v2/pokemon/" + strconv.Itoa(id)

	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Get(url)
	if err != nil {
		return Pokemon{}, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return Pokemon{}, err
	}

	var data struct {
		ID      int    `json:"id"`
		Name    string `json:"name"`
		Sprites struct {
			Front string `json:"front_default"`
		} `json:"sprites"`
		Types []struct {
			Type struct {
				Name string `json:"name"`
			} `json:"type"`
		} `json:"types"`
		Stats []struct {
			BaseStat int `json:"base_stat"`
			Stat     struct {
				Name string `json:"name"`
			} `json:"stat"`
		} `json:"stats"`
	}

	if err := json.Unmarshal(body, &data); err != nil {
		return Pokemon{}, err
	}

	p := Pokemon{
		ID:     data.ID,
		Name:   data.Name,
		Sprite: data.Sprites.Front,
	}

	for _, t := range data.Types {
		p.Types = append(p.Types, t.Type.Name)
	}

	for _, s := range data.Stats {
		switch s.Stat.Name {
		case "hp":
			p.HP = s.BaseStat
		case "attack":
			p.Attack = s.BaseStat
		}
	}

	return p, nil
}

// normalizeTarget ensures the target has a port
func normalizeTarget(target string) string {
	if !strings.Contains(target, ":") {
		return target + ":3000"
	}
	return target
}
