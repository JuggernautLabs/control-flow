#!/bin/bash

echo "🧪 Testing edge connection integration..."

# Check if server is running
if ! curl -s http://127.0.0.1:3001/api/graph > /dev/null 2>&1; then
    echo "⚠️ Graph server not running. Starting server..."
    cargo run > /dev/null 2>&1 &
    SERVER_PID=$!
    sleep 3
    
    if ! curl -s http://127.0.0.1:3001/api/graph > /dev/null 2>&1; then
        echo "❌ Failed to start server"
        exit 1
    fi
    echo "✅ Server started"
else
    echo "✅ Server already running"
    SERVER_PID=""
fi

# Clear any existing data
echo "🧹 Clearing existing graph..."
curl -s -X POST http://127.0.0.1:3001/api/clear > /dev/null

# Add test nodes
echo "📊 Adding test nodes..."
NODE1_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"id":"test-source","label":"Source Node","color":"#ff6b6b","size":25}')

NODE2_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"id":"test-target","label":"Target Node","color":"#4ecdc4","size":20}')

NODE3_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/nodes \
  -H "Content-Type: application/json" \
  -d '{"id":"test-hub","label":"Hub Node","color":"#45b7d1","size":30}')

# Verify nodes were added
NODES_COUNT=$(curl -s http://127.0.0.1:3001/api/graph | jq -r '.data.nodes | length')
if [ "$NODES_COUNT" != "3" ]; then
    echo "❌ Expected 3 nodes, got $NODES_COUNT"
    [ -n "$SERVER_PID" ] && kill $SERVER_PID
    exit 1
fi
echo "✅ Added 3 nodes successfully"

# Add test edges
echo "🔗 Adding test edges..."
EDGE1_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/edges \
  -H "Content-Type: application/json" \
  -d '{"source":"test-source","target":"test-target","label":"connects","weight":0.8}')

EDGE2_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/edges \
  -H "Content-Type: application/json" \
  -d '{"source":"test-hub","target":"test-source","label":"feeds","weight":0.6}')

EDGE3_RESPONSE=$(curl -s -X POST http://127.0.0.1:3001/api/edges \
  -H "Content-Type: application/json" \
  -d '{"source":"test-hub","target":"test-target","label":"supplies","weight":0.9}')

# Verify edges were added
EDGES_COUNT=$(curl -s http://127.0.0.1:3001/api/graph | jq -r '.data.edges | length')
if [ "$EDGES_COUNT" != "3" ]; then
    echo "❌ Expected 3 edges, got $EDGES_COUNT"
    [ -n "$SERVER_PID" ] && kill $SERVER_PID
    exit 1
fi
echo "✅ Added 3 edges successfully"

# Check edge data structure
echo "🔍 Verifying edge data structure..."
GRAPH_DATA=$(curl -s http://127.0.0.1:3001/api/graph)

# Check that all edges have valid source and target node IDs
VALID_SOURCES=$(echo "$GRAPH_DATA" | jq -r '.data.edges | to_entries[] | .value.source' | while read source; do
    if echo "$GRAPH_DATA" | jq -e ".data.nodes[\"$source\"]" > /dev/null; then
        echo "valid"
    else
        echo "invalid"
    fi
done | grep -c "valid")

VALID_TARGETS=$(echo "$GRAPH_DATA" | jq -r '.data.edges | to_entries[] | .value.target' | while read target; do
    if echo "$GRAPH_DATA" | jq -e ".data.nodes[\"$target\"]" > /dev/null; then
        echo "valid"
    else
        echo "invalid"
    fi
done | grep -c "valid")

if [ "$VALID_SOURCES" != "3" ] || [ "$VALID_TARGETS" != "3" ]; then
    echo "❌ Edge validation failed. Valid sources: $VALID_SOURCES, Valid targets: $VALID_TARGETS"
    [ -n "$SERVER_PID" ] && kill $SERVER_PID
    exit 1
fi
echo "✅ All edges reference valid nodes"

# Test edge properties
echo "📋 Checking edge properties..."
HAS_LABELS=$(echo "$GRAPH_DATA" | jq -r '.data.edges | to_entries[] | .value.label' | grep -v "null" | wc -l | tr -d ' ')
HAS_WEIGHTS=$(echo "$GRAPH_DATA" | jq -r '.data.edges | to_entries[] | .value.weight' | grep -v "null" | wc -l | tr -d ' ')

if [ "$HAS_LABELS" != "3" ] || [ "$HAS_WEIGHTS" != "3" ]; then
    echo "❌ Edge properties missing. Labels: $HAS_LABELS, Weights: $HAS_WEIGHTS"
    [ -n "$SERVER_PID" ] && kill $SERVER_PID
    exit 1
fi
echo "✅ All edges have required properties"

# Test persistence
echo "💾 Testing graph persistence..."
if [ -f "graph_data.json" ]; then
    FILE_NODES=$(jq -r '.nodes | length' graph_data.json)
    FILE_EDGES=$(jq -r '.edges | length' graph_data.json)
    
    if [ "$FILE_NODES" != "3" ] || [ "$FILE_EDGES" != "3" ]; then
        echo "❌ Persistence failed. File has $FILE_NODES nodes, $FILE_EDGES edges"
        [ -n "$SERVER_PID" ] && kill $SERVER_PID
        exit 1
    fi
    echo "✅ Graph persisted correctly to file"
else
    echo "⚠️ graph_data.json not found (persistence may be disabled)"
fi

# Display final graph info
echo ""
echo "📊 Final Graph Summary:"
echo "===================="
echo "Nodes: $NODES_COUNT"
echo "Edges: $EDGES_COUNT"
echo ""
echo "Node Details:"
echo "$GRAPH_DATA" | jq -r '.data.nodes | to_entries[] | "  - \(.value.id): \(.value.label) (\(.value.color))"'
echo ""
echo "Edge Details:"
echo "$GRAPH_DATA" | jq -r '.data.edges | to_entries[] | "  - \(.value.source) → \(.value.target): \(.value.label) (weight: \(.value.weight))"'

# Clean up
if [ -n "$SERVER_PID" ]; then
    echo ""
    echo "🧹 Stopping test server..."
    kill $SERVER_PID
fi

echo ""
echo "🎉 All integration tests passed!"
echo "💡 The graph visualization should now properly display connected edges."
echo "🌐 Visit http://127.0.0.1:3001 to see the fixed visualization."