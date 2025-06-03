// Simple ticket reader for the visualizer
const fs = require('fs');
const path = require('path');

async function getTicketFiles() {
  const ticketsDir = path.join(__dirname, '../../tickets');
  try {
    const files = await fs.promises.readdir(ticketsDir);
    const jsonFiles = files.filter(file => file.endsWith('.json'));
    
    const tickets = [];
    for (const file of jsonFiles) {
      const filePath = path.join(ticketsDir, file);
      const content = await fs.promises.readFile(filePath, 'utf8');
      const ticket = JSON.parse(content);
      ticket.id = file.replace('.json', '');
      ticket.filename = file;
      ticket.timestamp = ticket.timestamp || new Date(fs.statSync(filePath).mtime).toISOString();
      tickets.push(ticket);
    }
    
    return tickets.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
  } catch (error) {
    console.error('Error reading tickets:', error);
    return [];
  }
}

module.exports = { getTicketFiles };