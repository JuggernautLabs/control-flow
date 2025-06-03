// Simple HTTP server to serve tickets and the visualizer
const http = require('http');
const fs = require('fs');
const path = require('path');
const { getTicketFiles } = require('./src/utils/ticket-reader.cjs');

const PORT = 3001;

// Adventure game state storage
const adventureStates = new Map();
const adventureStats = {
  totalPlayers: 0,
  choicesMade: 0,
  completionRate: 0,
  popularChoices: new Map()
};

const server = http.createServer(async (req, res) => {
  // Enable CORS
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  
  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }
  
  const url = new URL(req.url, `http://localhost:${PORT}`);
  
  if (url.pathname === '/api/tickets') {
    try {
      const tickets = await getTicketFiles();
      res.setHeader('Content-Type', 'application/json');
      res.writeHead(200);
      res.end(JSON.stringify(tickets));
    } catch (error) {
      res.writeHead(500);
      res.end(JSON.stringify({ error: 'Failed to load tickets' }));
    }
  } else if (url.pathname === '/api/adventure/track') {
    if (req.method === 'POST') {
      let body = '';
      req.on('data', chunk => {
        body += chunk.toString();
      });
      req.on('end', () => {
        try {
          const data = JSON.parse(body);
          
          // Generate session ID if not provided
          const sessionId = data.sessionId || `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
          
          // Update adventure stats
          adventureStats.choicesMade++;
          if (adventureStats.popularChoices.has(data.choiceId)) {
            adventureStats.popularChoices.set(data.choiceId, adventureStats.popularChoices.get(data.choiceId) + 1);
          } else {
            adventureStats.popularChoices.set(data.choiceId, 1);
          }
          
          // Store player state
          adventureStates.set(sessionId, {
            ...data,
            timestamp: new Date().toISOString(),
            sessionId
          });
          
          console.log(`📊 Adventure choice tracked: ${data.choiceId} (${data.fromNode} → ${data.toNode})`);
          
          res.setHeader('Content-Type', 'application/json');
          res.writeHead(200);
          res.end(JSON.stringify({ 
            success: true, 
            sessionId,
            totalChoices: adventureStats.choicesMade,
            message: 'Progress tracked successfully'
          }));
        } catch (error) {
          res.writeHead(400);
          res.end(JSON.stringify({ error: 'Invalid JSON data' }));
        }
      });
    } else {
      res.writeHead(405);
      res.end(JSON.stringify({ error: 'Method not allowed' }));
    }
  } else if (url.pathname === '/api/adventure/stats') {
    const stats = {
      ...adventureStats,
      popularChoices: Array.from(adventureStats.popularChoices.entries())
        .sort(([,a], [,b]) => b - a)
        .slice(0, 10)
        .map(([choice, count]) => ({ choice, count })),
      activeSessions: adventureStates.size,
      recentSessions: Array.from(adventureStates.values())
        .sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp))
        .slice(0, 5)
    };
    
    res.setHeader('Content-Type', 'application/json');
    res.writeHead(200);
    res.end(JSON.stringify(stats));
  } else if (url.pathname === '/api/adventure/leaderboard') {
    const leaderboard = Array.from(adventureStates.values())
      .filter(state => state.gameState)
      .sort((a, b) => {
        // Sort by level first, then by experience
        if (b.gameState.level !== a.gameState.level) {
          return b.gameState.level - a.gameState.level;
        }
        return b.gameState.experience - a.gameState.experience;
      })
      .slice(0, 10)
      .map((state, index) => ({
        rank: index + 1,
        sessionId: state.sessionId.substr(-8), // Show last 8 characters for anonymity
        level: state.gameState.level,
        experience: state.gameState.experience,
        gold: state.gameState.gold,
        lastActive: state.timestamp
      }));
    
    res.setHeader('Content-Type', 'application/json');
    res.writeHead(200);
    res.end(JSON.stringify(leaderboard));
  } else if (url.pathname === '/' || url.pathname === '/visualizer') {
    try {
      const htmlPath = path.join(__dirname, 'src/pages/visualizer.html');
      const html = fs.readFileSync(htmlPath, 'utf8');
      
      // Update the HTML to use the API
      const updatedHtml = html.replace(
        'async loadTickets() {',
        `async loadTickets() {
          try {
            const response = await fetch('http://localhost:${PORT}/api/tickets');
            if (!response.ok) throw new Error('Failed to fetch tickets');
            this.tickets = await response.json();
            this.loading = false;
            return;
          } catch (err) {
            console.error('API fetch failed, using sample data:', err);
          }
          // Original code as fallback:`
      );
      
      res.setHeader('Content-Type', 'text/html');
      res.writeHead(200);
      res.end(updatedHtml);
    } catch (error) {
      res.writeHead(500);
      res.end('Error loading visualizer');
    }
  } else {
    res.writeHead(404);
    res.end('Not found');
  }
});

server.listen(PORT, () => {
  console.log(`🎯 Control Flow Server running at:`);
  console.log(`   http://localhost:${PORT}/`);
  console.log(`\\n📊 Available APIs:`);
  console.log(`   http://localhost:${PORT}/api/tickets - Cognitive analysis tickets`);
  console.log(`   http://localhost:${PORT}/api/adventure/track - Adventure progress tracking`);
  console.log(`   http://localhost:${PORT}/api/adventure/stats - Adventure statistics`);
  console.log(`   http://localhost:${PORT}/api/adventure/leaderboard - Player leaderboard`);
  console.log(`\\n🎮 Adventure game endpoints ready for interactive demos!`);
});

module.exports = server;