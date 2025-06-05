#!/usr/bin/env node

// Node.js test runner for D3.js simulation logic
// This tests the core logic without requiring a browser

const assert = require('assert');

// Mock D3.js force simulation behavior for testing
class MockForceSimulation {
    constructor() {
        this.nodeData = [];
        this.linkData = [];
        this.forces = {};
        this.tickCallbacks = [];
        this.alphaValue = 1.0;
    }

    nodes(data) {
        if (data) {
            this.nodeData = data;
            return this;
        }
        return this.nodeData;
    }

    force(name, force) {
        if (force) {
            this.forces[name] = force;
            return this;
        }
        return this.forces[name];
    }

    on(event, callback) {
        if (event === 'tick') {
            this.tickCallbacks.push(callback);
        }
        return this;
    }

    alpha(value) {
        if (value !== undefined) {
            this.alphaValue = value;
            return this;
        }
        return this.alphaValue;
    }

    restart() {
        // Simulate D3's ID conversion behavior
        this.convertStringIdsToObjects();
        
        // Simulate a few ticks
        for (let i = 0; i < 5; i++) {
            this.simulateTick();
        }
        return this;
    }

    convertStringIdsToObjects() {
        // This mimics D3's forceLink behavior of converting string IDs to object references
        const linkForce = this.forces.link;
        if (linkForce && linkForce.links) {
            const links = linkForce.links();
            links.forEach(link => {
                if (typeof link.source === 'string') {
                    const sourceNode = this.nodeData.find(n => n.id === link.source);
                    if (sourceNode) {
                        link.source = sourceNode;
                    }
                }
                if (typeof link.target === 'string') {
                    const targetNode = this.nodeData.find(n => n.id === link.target);
                    if (targetNode) {
                        link.target = targetNode;
                    }
                }
            });
        }
    }

    simulateTick() {
        // Update node positions randomly to simulate physics
        this.nodeData.forEach(node => {
            if (!node.x) node.x = 300 + (Math.random() - 0.5) * 100;
            if (!node.y) node.y = 150 + (Math.random() - 0.5) * 100;
            
            // Small random movement
            node.x += (Math.random() - 0.5) * 2;
            node.y += (Math.random() - 0.5) * 2;
        });

        // Call tick callbacks
        this.tickCallbacks.forEach(callback => callback());
    }
}

class MockForceLink {
    constructor() {
        this.linkData = [];
        this.idAccessor = d => d.id;
    }

    id(accessor) {
        if (accessor) {
            this.idAccessor = accessor;
            return this;
        }
        return this.idAccessor;
    }

    links(data) {
        if (data) {
            this.linkData = data;
            return this;
        }
        return this.linkData;
    }

    distance() { return this; }
    strength() { return this; }
}

// Test cases
console.log('🧪 Running D3.js simulation tests...\n');

function testBasicSimulation() {
    console.log('Test 1: Basic simulation setup');
    
    const nodes = [
        { id: 'node1', label: 'Node 1' },
        { id: 'node2', label: 'Node 2' }
    ];
    
    const links = [
        { id: 'edge1', source: 'node1', target: 'node2' }
    ];
    
    const simulation = new MockForceSimulation();
    const linkForce = new MockForceLink();
    
    simulation.force('link', linkForce);
    simulation.nodes(nodes);
    linkForce.links(links);
    
    // Before restart, links should have string IDs
    assert.strictEqual(typeof links[0].source, 'string');
    assert.strictEqual(typeof links[0].target, 'string');
    
    simulation.restart();
    
    // After restart, links should have object references
    assert.strictEqual(typeof links[0].source, 'object');
    assert.strictEqual(typeof links[0].target, 'object');
    assert.strictEqual(links[0].source.id, 'node1');
    assert.strictEqual(links[0].target.id, 'node2');
    
    console.log('✅ Basic simulation test passed\n');
}

function testEdgePositioning() {
    console.log('Test 2: Edge positioning logic');
    
    const nodes = [
        { id: 'a', label: 'A', x: 100, y: 50 },
        { id: 'b', label: 'B', x: 200, y: 150 }
    ];
    
    const links = [
        { id: 'edge1', source: 'a', target: 'b' }
    ];
    
    const simulation = new MockForceSimulation();
    const linkForce = new MockForceLink();
    
    simulation.force('link', linkForce);
    simulation.nodes(nodes);
    linkForce.links(links);
    
    let edgeCoordinates = { x1: 0, y1: 0, x2: 0, y2: 0 };
    
    // Mock tick handler (same logic as in the main app)
    simulation.on('tick', () => {
        const link = links[0];
        
        if (typeof link.source === 'string') {
            const sourceNode = nodes.find(n => n.id === link.source);
            edgeCoordinates.x1 = sourceNode ? sourceNode.x : 300;
            edgeCoordinates.y1 = sourceNode ? sourceNode.y : 150;
        } else {
            edgeCoordinates.x1 = link.source && link.source.x !== undefined ? link.source.x : 300;
            edgeCoordinates.y1 = link.source && link.source.y !== undefined ? link.source.y : 150;
        }
        
        if (typeof link.target === 'string') {
            const targetNode = nodes.find(n => n.id === link.target);
            edgeCoordinates.x2 = targetNode ? targetNode.x : 300;
            edgeCoordinates.y2 = targetNode ? targetNode.y : 150;
        } else {
            edgeCoordinates.x2 = link.target && link.target.x !== undefined ? link.target.x : 300;
            edgeCoordinates.y2 = link.target && link.target.y !== undefined ? link.target.y : 150;
        }
    });
    
    simulation.restart();
    
    // Edge should connect to node positions, not center
    console.log(`Edge coordinates: (${edgeCoordinates.x1},${edgeCoordinates.y1}) to (${edgeCoordinates.x2},${edgeCoordinates.y2})`);
    
    // Should not be at center (300, 150)
    const notCentered = !(edgeCoordinates.x1 === 300 && edgeCoordinates.y1 === 150) &&
                       !(edgeCoordinates.x2 === 300 && edgeCoordinates.y2 === 150);
    
    assert(notCentered, 'Edges should not cluster at center');
    assert(edgeCoordinates.x1 !== edgeCoordinates.x2 || edgeCoordinates.y1 !== edgeCoordinates.y2, 
           'Edge endpoints should be different');
    
    console.log('✅ Edge positioning test passed\n');
}

function testMultipleEdges() {
    console.log('Test 3: Multiple edges from same node');
    
    const nodes = [
        { id: 'hub', label: 'Hub' },
        { id: 'a', label: 'A' },
        { id: 'b', label: 'B' },
        { id: 'c', label: 'C' }
    ];
    
    const links = [
        { id: 'edge1', source: 'hub', target: 'a' },
        { id: 'edge2', source: 'hub', target: 'b' },
        { id: 'edge3', source: 'hub', target: 'c' }
    ];
    
    const simulation = new MockForceSimulation();
    const linkForce = new MockForceLink();
    
    simulation.force('link', linkForce);
    simulation.nodes(nodes);
    linkForce.links(links);
    
    simulation.restart();
    
    // All links should have the same source (hub) but different targets
    const sourceIds = links.map(l => l.source.id);
    const targetIds = links.map(l => l.target.id);
    
    assert(sourceIds.every(id => id === 'hub'), 'All edges should have hub as source');
    
    const uniqueTargets = new Set(targetIds);
    assert.strictEqual(uniqueTargets.size, 3, 'All targets should be different');
    
    console.log('✅ Multiple edges test passed\n');
}

function testErrorHandling() {
    console.log('Test 4: Error handling for missing nodes');
    
    const nodes = [
        { id: 'existing', label: 'Existing Node' }
    ];
    
    const links = [
        { id: 'edge1', source: 'existing', target: 'nonexistent' }
    ];
    
    const simulation = new MockForceSimulation();
    const linkForce = new MockForceLink();
    
    simulation.force('link', linkForce);
    simulation.nodes(nodes);
    linkForce.links(links);
    
    let edgeCoordinates = { x1: 0, y1: 0, x2: 0, y2: 0 };
    
    // Mock tick handler with fallback logic
    simulation.on('tick', () => {
        const link = links[0];
        
        if (typeof link.source === 'string') {
            const sourceNode = nodes.find(n => n.id === link.source);
            edgeCoordinates.x1 = sourceNode ? sourceNode.x : 300; // fallback to center
        } else {
            edgeCoordinates.x1 = link.source && link.source.x !== undefined ? link.source.x : 300;
        }
        
        if (typeof link.target === 'string') {
            const targetNode = nodes.find(n => n.id === link.target);
            edgeCoordinates.x2 = targetNode ? targetNode.x : 300; // fallback to center
        } else {
            edgeCoordinates.x2 = link.target && link.target.x !== undefined ? link.target.x : 300;
        }
    });
    
    simulation.restart();
    
    // For missing target, should fallback to center
    assert.strictEqual(edgeCoordinates.x2, 300, 'Missing target should fallback to center');
    
    console.log('✅ Error handling test passed\n');
}

// Run all tests
try {
    testBasicSimulation();
    testEdgePositioning();
    testMultipleEdges();
    testErrorHandling();
    
    console.log('🎉 All tests passed! The edge connection logic should work correctly.');
    process.exit(0);
} catch (error) {
    console.error('❌ Test failed:', error.message);
    process.exit(1);
}