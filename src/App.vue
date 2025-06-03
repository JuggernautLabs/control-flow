<template>
  <div id="app">
    <div class="container">
      <h1>🎯 Control Flow</h1>
      
      <div class="nav-links">
        <router-link to="/" class="nav-link">🏠 Home</router-link>
        <router-link to="/interactive" class="nav-link">🎮 Interactive</router-link>
        <router-link to="/graph" class="nav-link">🕸️ Dependency Graph</router-link>
        <router-link to="/tickets" class="nav-link">📋 Ticket List</router-link>
        <a href="http://localhost:3001" class="nav-link" target="_blank">🚀 Launch Server</a>
      </div>

      <router-view />
    </div>
  </div>
</template>

<script>
export default {
  name: 'App'
}
</script>

<style>
body {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  margin: 0;
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  min-height: 100vh;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  background: white;
  border-radius: 12px;
  padding: 30px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.2);
}

h1 {
  color: #333;
  text-align: center;
  margin-bottom: 30px;
  font-size: 2.5em;
  background: linear-gradient(45deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-links {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-bottom: 30px;
}

.nav-link {
  padding: 10px 20px;
  background: #667eea;
  color: white;
  text-decoration: none;
  border-radius: 6px;
  transition: background 0.3s ease;
}

.nav-link:hover {
  background: #764ba2;
}

.description {
  text-align: center;
  color: #666;
  font-size: 1.1em;
  line-height: 1.6;
  margin-bottom: 30px;
}
</style>