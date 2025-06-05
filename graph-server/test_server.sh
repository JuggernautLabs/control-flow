#!/bin/bash

echo "Starting graph server..."
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo "Testing API endpoints..."

# Test graph endpoint
echo "GET /api/graph:"
curl -s http://127.0.0.1:3001/api/graph | jq '.'

# Test adding a node
echo -e "\nPOST /api/nodes:"
curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"label":"Test Node","color":"#ff6b6b"}' | jq '.'

# Test graph after adding node
echo -e "\nGET /api/graph after adding node:"
curl -s http://127.0.0.1:3001/api/graph | jq '.'

# Kill server
kill $SERVER_PID
echo -e "\nServer stopped."