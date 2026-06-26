// examples/minimal_client.go
//
// This is a minimal Golang client demonstrating how to connect to the 
// Extended Markdown IPC daemon. It pushes Markdown updates via POST
// and listens to Server-Sent Events (SSE) for UI interactions.

package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"
)

const IPC_URL = "http://127.0.0.1:3030"
const SEC_TOKEN = "super_secret_token_123"

func main() {
	fmt.Println("Starting Golang Minimal Client...")

	// 1. Run a goroutine to push an update
	go func() {
		time.Sleep(2 * time.Second)
		fmt.Println("\n[IPC PUSH] Sending dynamic markdown update...")

		markdownContent := "# Live Golang Update\n\nInjected by Go!\n\n```plot\n{\"type\":\"line\"}\n```"
		
		req, err := http.NewRequest("POST", IPC_URL+"/update", bytes.NewBuffer([]byte(markdownContent)))
		if err != nil {
			fmt.Println("Failed to create request:", err)
			return
		}
		
		req.Header.Set("x-sec-token", SEC_TOKEN)
		req.Header.Set("Content-Type", "text/plain")

		client := &http.Client{}
		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("[IPC PUSH ERROR]", err)
			return
		}
		defer resp.Body.Close()
		fmt.Println("[IPC PUSH SUCCESS] Response Status:", resp.Status)
	}()

	// 2. Listen for UI Events via SSE
	fmt.Println("Connecting to IPC Event Stream...")

	req, err := http.NewRequest("GET", IPC_URL+"/events", nil)
	if err != nil {
		panic(err)
	}
	
	req.Header.Set("x-sec-token", SEC_TOKEN)
	req.Header.Set("Accept", "text/event-stream")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		fmt.Println("Unauthorized or error connecting to stream:", resp.Status)
		return
	}

	reader := bufio.NewReader(resp.Body)
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println("Stream closed or error:", err)
			break
		}

		if strings.HasPrefix(line, "data: ") {
			jsonStr := strings.TrimSpace(strings.TrimPrefix(line, "data: "))
			if jsonStr == "" {
				continue
			}

			var event map[string]interface{}
			if err := json.Unmarshal([]byte(jsonStr), &event); err == nil {
				fmt.Println("\n[IPC EVENT RECEIVED] 🔔")
				fmt.Println("Type:", event["event_type"])
				fmt.Println("Element:", event["element_id"])
				fmt.Println("Payload:", event["payload"])
			}
		}
	}
}
