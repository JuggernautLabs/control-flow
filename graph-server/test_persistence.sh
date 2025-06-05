#!/bin/bash

echo "Testing graph persistence and auto-save..."

# Remove any existing save file
rm -f graph_data.json

# Start server
echo "Starting server..."
cargo run &
SERVER_PID=$!
sleep 3

echo "Adding test nodes..."
# Add a few nodes
curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"label":"Requirement A","color":"#ff6b6b","size":25}' > /dev/null

curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"label":"Implementation B","color":"#4ecdc4","size":20}' > /dev/null

echo "Checking if graph_data.json was created..."
if [ -f "graph_data.json" ]; then
    echo "✅ graph_data.json exists!"
    echo "Content preview:"
    head -10 graph_data.json
else
    echo "❌ graph_data.json was not created"
fi

echo "Getting current graph state..."
curl -s http://127.0.0.1:3001/api/graph | jq '.data.nodes | length' | xargs echo "Nodes in memory:"

# Kill server
kill $SERVER_PID
echo "Server stopped."

echo -e "\nRestarting server to test persistence..."
cargo run &
SERVER_PID=$!
sleep 3

echo "Checking if nodes were restored..."
curl -s http://127.0.0.1:3001/api/graph | jq '.data.nodes | length' | xargs echo "Nodes after restart:"

# Clean up
kill $SERVER_PID
echo "Test completed."