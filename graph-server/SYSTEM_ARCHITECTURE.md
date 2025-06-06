# Graph Visualization System Summary

## System Architecture

This is a **loosely coupled** client-server graph visualization system built with:
- **Backend**: Rust (Axum framework) providing RESTful API
- **Frontend**: Vanilla JavaScript with D3.js for visualization
- **Storage**: JSON persistence + browser localStorage for projects

## Core Data Model

### Node Structure
```typescript
interface Node {
    id: string;           // Unique identifier
    label: string;        // Display name
    color?: string;       // Visual styling
    size?: number;        // Radius in pixels
    metadata: HashMap<String, String>  // Arbitrary key-value pairs
}
```

### Edge Structure
```typescript
interface Edge {
    id: string;           // Unique identifier
    source: string;       // Source node ID
    target: string;       // Target node ID
    label?: string;       // Display text
    weight?: number;      // Influence strength
    color?: string;       // Visual styling
    metadata: HashMap<String, String>  // Arbitrary key-value pairs
}
```

## Special Metadata Treatment

**Currently, there are NO special metadata fields with hardcoded behaviors.** The system treats all metadata as generic key-value pairs that are:
- Stored as strings in the backend
- Displayed in the side menu when nodes are selected
- Preserved during project save/load operations
- Included in export/import functionality

The metadata system is **completely generic** - no fields trigger special rendering, behavior, or validation.

## System Coupling Analysis

### 🔗 **Loose Coupling Characteristics**

#### **API Layer Separation**
- Backend provides pure REST API (`/api/graph`, `/api/nodes`, `/api/edges`)
- Frontend can be completely replaced without touching backend
- No shared code or tight dependencies between layers

#### **Data Format Independence**
- JSON over HTTP - platform agnostic
- Backend has no knowledge of frontend visualization choices
- Frontend has no knowledge of backend storage implementation

#### **Component Modularity**
```javascript
// Visualization is self-contained
class SimpleGraphRenderer { ... }

// API calls are abstracted
async function apiCall(url, options) { ... }

// Project management is separate module
function saveAsNewProject() { ... }
```

### 🔗 **Tight Coupling Areas**

#### **D3.js Physics Integration**
The renderer has **tight coupling** with D3.js force simulation:
```javascript
// Tightly coupled to D3's data transformation
this.simulation.nodes(nodes);
this.simulation.force('link').links(links);

// D3 mutates data objects directly
links.forEach(link => {
    // D3 transforms string IDs to object references
    // link.source becomes node object
    // link.target becomes node object
});
```

#### **DOM Manipulation Dependencies**
Frontend is **tightly coupled** to specific HTML structure:
```javascript
// Assumes specific DOM IDs exist
document.getElementById('nodeDetailId')
document.getElementById('sideMenu')
document.getElementById('projectSelector')
```

#### **State Management Coupling**
```javascript
// Global state creates coupling between components
let selectedNodeId = null;
let graphData = { nodes: {}, edges: {} };
let currentProjectName = 'Default Project';

// Functions directly modify global state
function navigateToNode(nodeId) {
    selectedNodeId = nodeId;  // Direct global mutation
}
```

### 🔗 **Interface Contracts**

#### **Backend API Contract**
```json
{
    "success": boolean,
    "data": T | null,
    "error": string | null
}
```

#### **Frontend Expectations**
- Backend must preserve node/edge IDs across operations
- Physics simulation expects D3-compatible data format
- Project system expects serializable JSON structure

## Data Flow Architecture

### **Request Flow**
```
User Action → Frontend Function → API Call → Backend Logic → JSON Response → UI Update
```

### **Visualization Pipeline**
```
Graph Data → D3 Simulation → SVG Rendering → User Interaction → State Update
```

### **Project Management Flow**
```
Graph State → Serialize Project → localStorage/Export → Import/Load → Restore State
```

## Extensibility Points

### ✅ **Easy to Extend**
- **New node/edge properties**: Just add to metadata
- **Additional API endpoints**: Backend is stateless REST
- **New visualization features**: D3.js is highly flexible
- **Different storage backends**: Change backend without affecting frontend

### ⚠️ **Moderate Complexity**
- **New metadata behaviors**: Requires frontend code changes
- **Different physics engines**: Would require renderer rewrite
- **Alternative data formats**: Would need API contract changes

### ❌ **Difficult to Change**
- **Non-JSON data structures**: Breaks project import/export
- **Different frontend framework**: Global state management needs redesign
- **Real-time collaboration**: Would require significant architectural changes

## System Strengths

1. **Clear separation of concerns** between visualization and data management
2. **Stateless backend** makes scaling and testing easier  
3. **Generic metadata system** allows arbitrary data without code changes
4. **Project management** provides good user workflow
5. **Smart update system** preserves user arrangements during data changes

## System Weaknesses

1. **Global state management** creates coupling between UI components
2. **No real-time capabilities** - requires manual refresh for multi-user scenarios
3. **Browser localStorage dependency** for project persistence
4. **Tight D3.js coupling** makes renderer replacement difficult
5. **No validation layer** for metadata or graph structure constraints

## Conclusion

This is a **moderately coupled** system with clean API boundaries but tight internal coupling within the frontend visualization layer. The generic metadata approach and REST API design provide good extensibility, while the D3.js integration and global state management create some architectural constraints.