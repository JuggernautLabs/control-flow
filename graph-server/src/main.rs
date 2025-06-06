use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub color: Option<String>,
    pub size: Option<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub weight: Option<f64>,
    pub color: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    
    fn load_from_file(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<Graph>(&content) {
                    Ok(graph) => {
                        info!("Loaded graph from file: {} nodes, {} edges", 
                              graph.nodes.len(), graph.edges.len());
                        graph
                    }
                    Err(e) => {
                        warn!("Failed to parse graph file: {}", e);
                        Self::new()
                    }
                }
            }
            Err(_) => {
                info!("No existing graph file found, starting with empty graph");
                Self::new()
            }
        }
    }
    
    fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                match fs::write(path, content) {
                    Ok(()) => {
                        info!("Saved graph to file: {} nodes, {} edges", 
                              self.nodes.len(), self.edges.len());
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to write graph file: {}", e);
                        Err(format!("Failed to write file: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize graph: {}", e);
                Err(format!("Failed to serialize graph: {}", e))
            }
        }
    }

    fn add_node(&mut self, node: Node) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("Node with id '{}' already exists", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    fn add_edge(&mut self, edge: Edge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(format!("Source node '{}' does not exist", edge.source));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(format!("Target node '{}' does not exist", edge.target));
        }
        if self.edges.contains_key(&edge.id) {
            return Err(format!("Edge with id '{}' already exists", edge.id));
        }
        self.edges.insert(edge.id.clone(), edge);
        Ok(())
    }

    fn remove_node(&mut self, node_id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(node_id) {
            return Err(format!("Node '{}' does not exist", node_id));
        }
        
        // Remove all edges connected to this node
        self.edges.retain(|_, edge| edge.source != node_id && edge.target != node_id);
        
        // Remove the node
        self.nodes.remove(node_id);
        Ok(())
    }

    fn remove_edge(&mut self, edge_id: &str) -> Result<(), String> {
        if !self.edges.contains_key(edge_id) {
            return Err(format!("Edge '{}' does not exist", edge_id));
        }
        self.edges.remove(edge_id);
        Ok(())
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}

struct GraphState {
    graph: Graph,
    save_path: PathBuf,
    projects_path: PathBuf,
}

impl GraphState {
    fn new(save_path: PathBuf) -> Self {
        let graph = Graph::load_from_file(&save_path);
        let projects_path = PathBuf::from("projects");
        
        // Create projects directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&projects_path) {
            warn!("Failed to create projects directory: {}", e);
        }
        
        Self { graph, save_path, projects_path }
    }
    
    fn save(&self) -> Result<(), String> {
        self.graph.save_to_file(&self.save_path)
    }
    
    fn save_project(&self, project_data: &ProjectData) -> Result<(), String> {
        let project_file = self.projects_path.join(format!("{}.json", 
            project_data.name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")));
        
        match serde_json::to_string_pretty(project_data) {
            Ok(content) => {
                match fs::write(&project_file, content) {
                    Ok(()) => {
                        info!("Saved project '{}' to file: {:?}", project_data.name, project_file);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to write project file: {}", e);
                        Err(format!("Failed to write project file: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize project: {}", e);
                Err(format!("Failed to serialize project: {}", e))
            }
        }
    }
    
    fn load_project(&self, project_name: &str) -> Result<ProjectData, String> {
        let project_file = self.projects_path.join(format!("{}.json", 
            project_name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")));
        
        match fs::read_to_string(&project_file) {
            Ok(content) => {
                match serde_json::from_str::<ProjectData>(&content) {
                    Ok(project) => {
                        info!("Loaded project '{}' from file: {:?}", project_name, project_file);
                        Ok(project)
                    }
                    Err(e) => {
                        error!("Failed to parse project file: {}", e);
                        Err(format!("Failed to parse project file: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to read project file: {}", e);
                Err(format!("Project '{}' not found", project_name))
            }
        }
    }
    
    fn list_projects(&self) -> Result<Vec<String>, String> {
        match fs::read_dir(&self.projects_path) {
            Ok(entries) => {
                let mut projects = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".json") {
                                let project_name = filename.trim_end_matches(".json").to_string();
                                projects.push(project_name);
                            }
                        }
                    }
                }
                projects.sort();
                Ok(projects)
            }
            Err(e) => {
                warn!("Failed to read projects directory: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    fn delete_project(&self, project_name: &str) -> Result<(), String> {
        let project_file = self.projects_path.join(format!("{}.json", 
            project_name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")));
        
        match fs::remove_file(&project_file) {
            Ok(()) => {
                info!("Deleted project '{}': {:?}", project_name, project_file);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete project file: {}", e);
                Err(format!("Failed to delete project: {}", e))
            }
        }
    }
}

type SharedGraphState = Arc<RwLock<GraphState>>;

#[derive(Deserialize)]
struct AddNodeRequest {
    id: Option<String>,
    label: String,
    color: Option<String>,
    size: Option<f64>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct AddEdgeRequest {
    id: Option<String>,
    source: String,
    target: String,
    label: Option<String>,
    weight: Option<f64>,
    color: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
    pub config: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: String,
}

#[derive(Deserialize)]
struct SaveProjectRequest {
    name: String,
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    config: Option<HashMap<String, serde_json::Value>>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

async fn get_graph(State(graph_state): State<SharedGraphState>) -> Json<ApiResponse<Graph>> {
    let graph = graph_state.read().unwrap().graph.clone();
    Json(ApiResponse::success(graph))
}

async fn add_node(
    State(graph_state): State<SharedGraphState>,
    Json(req): Json<AddNodeRequest>,
) -> Result<Json<ApiResponse<Node>>, StatusCode> {
    let node = Node {
        id: req.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        label: req.label,
        color: req.color,
        size: req.size,
        metadata: req.metadata.unwrap_or_default(),
    };

    let mut state = graph_state.write().unwrap();
    match state.graph.add_node(node.clone()) {
        Ok(()) => {
            info!("Added node: {}", node.id);
            if let Err(e) = state.save() {
                warn!("Failed to save graph after adding node: {}", e);
            }
            Ok(Json(ApiResponse::success(node)))
        }
        Err(e) => {
            warn!("Failed to add node: {}", e);
            Ok(Json(ApiResponse::error(e)))
        }
    }
}

async fn add_edge(
    State(graph_state): State<SharedGraphState>,
    Json(req): Json<AddEdgeRequest>,
) -> Result<Json<ApiResponse<Edge>>, StatusCode> {
    let edge = Edge {
        id: req.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        source: req.source,
        target: req.target,
        label: req.label,
        weight: req.weight,
        color: req.color,
        metadata: req.metadata.unwrap_or_default(),
    };

    let mut state = graph_state.write().unwrap();
    match state.graph.add_edge(edge.clone()) {
        Ok(()) => {
            info!("Added edge: {} -> {}", edge.source, edge.target);
            if let Err(e) = state.save() {
                warn!("Failed to save graph after adding edge: {}", e);
            }
            Ok(Json(ApiResponse::success(edge)))
        }
        Err(e) => {
            warn!("Failed to add edge: {}", e);
            Ok(Json(ApiResponse::error(e)))
        }
    }
}

async fn remove_node(
    State(graph_state): State<SharedGraphState>,
    Path(node_id): Path<String>,
) -> Json<ApiResponse<String>> {
    let mut state = graph_state.write().unwrap();
    match state.graph.remove_node(&node_id) {
        Ok(()) => {
            info!("Removed node: {}", node_id);
            if let Err(e) = state.save() {
                warn!("Failed to save graph after removing node: {}", e);
            }
            Json(ApiResponse::success(format!("Node '{}' removed", node_id)))
        }
        Err(e) => {
            warn!("Failed to remove node: {}", e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn remove_edge(
    State(graph_state): State<SharedGraphState>,
    Path(edge_id): Path<String>,
) -> Json<ApiResponse<String>> {
    let mut state = graph_state.write().unwrap();
    match state.graph.remove_edge(&edge_id) {
        Ok(()) => {
            info!("Removed edge: {}", edge_id);
            if let Err(e) = state.save() {
                warn!("Failed to save graph after removing edge: {}", e);
            }
            Json(ApiResponse::success(format!("Edge '{}' removed", edge_id)))
        }
        Err(e) => {
            warn!("Failed to remove edge: {}", e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn clear_graph(State(graph_state): State<SharedGraphState>) -> Json<ApiResponse<String>> {
    let mut state = graph_state.write().unwrap();
    state.graph.clear();
    if let Err(e) = state.save() {
        warn!("Failed to save graph after clearing: {}", e);
    }
    info!("Graph cleared");
    Json(ApiResponse::success("Graph cleared".to_string()))
}

async fn save_project(
    State(graph_state): State<SharedGraphState>,
    Json(req): Json<SaveProjectRequest>,
) -> Json<ApiResponse<String>> {
    let project_data = ProjectData {
        name: req.name.clone(),
        nodes: req.nodes,
        edges: req.edges,
        config: req.config,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };
    
    let state = graph_state.read().unwrap();
    match state.save_project(&project_data) {
        Ok(()) => {
            info!("Project '{}' saved successfully", project_data.name);
            Json(ApiResponse::success(format!("Project '{}' saved successfully", project_data.name)))
        }
        Err(e) => {
            warn!("Failed to save project '{}': {}", project_data.name, e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn load_project(
    State(graph_state): State<SharedGraphState>,
    Path(project_name): Path<String>,
) -> Json<ApiResponse<ProjectData>> {
    let state = graph_state.read().unwrap();
    match state.load_project(&project_name) {
        Ok(project) => {
            info!("Project '{}' loaded successfully", project_name);
            Json(ApiResponse::success(project))
        }
        Err(e) => {
            warn!("Failed to load project '{}': {}", project_name, e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn list_projects(State(graph_state): State<SharedGraphState>) -> Json<ApiResponse<Vec<String>>> {
    let state = graph_state.read().unwrap();
    match state.list_projects() {
        Ok(projects) => {
            Json(ApiResponse::success(projects))
        }
        Err(e) => {
            warn!("Failed to list projects: {}", e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn delete_project(
    State(graph_state): State<SharedGraphState>,
    Path(project_name): Path<String>,
) -> Json<ApiResponse<String>> {
    let mut state = graph_state.write().unwrap();
    match state.delete_project(&project_name) {
        Ok(()) => {
            info!("Project '{}' deleted successfully", project_name);
            Json(ApiResponse::success(format!("Project '{}' deleted successfully", project_name)))
        }
        Err(e) => {
            warn!("Failed to delete project '{}': {}", project_name, e);
            Json(ApiResponse::error(e))
        }
    }
}

async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn serve_test() -> Html<&'static str> {
    Html(include_str!("../static/test-basic.html"))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let save_path = PathBuf::from("graph_data.json");
    let graph_state = Arc::new(RwLock::new(GraphState::new(save_path)));

    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/test", get(serve_test))
        .route("/api/graph", get(get_graph))
        .route("/api/nodes", post(add_node))
        .route("/api/edges", post(add_edge))
        .route("/api/nodes/:id", delete(remove_node))
        .route("/api/edges/:id", delete(remove_edge))
        .route("/api/clear", post(clear_graph))
        .route("/api/projects", get(list_projects))
        .route("/api/projects", post(save_project))
        .route("/api/projects/:name", get(load_project))
        .route("/api/projects/:name", delete(delete_project))
        .layer(CorsLayer::permissive())
        .with_state(graph_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    info!("Graph server running on http://127.0.0.1:3001");
    info!("Graph data will be saved to: graph_data.json");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use tempfile::TempDir;
    use serde_json::json;

    fn create_test_app() -> (Router, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("test_graph.json");
        let graph_state = Arc::new(RwLock::new(GraphState::new(save_path)));

        let app = Router::new()
            .route("/api/graph", get(get_graph))
            .route("/api/nodes", post(add_node))
            .route("/api/edges", post(add_edge))
            .route("/api/nodes/:id", delete(remove_node))
            .route("/api/edges/:id", delete(remove_edge))
            .route("/api/clear", post(clear_graph))
            .layer(CorsLayer::permissive())
            .with_state(graph_state);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_empty_graph() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/api/graph").await;
        response.assert_status_ok();
        
        let graph: ApiResponse<Graph> = response.json();
        assert!(graph.success);
        assert_eq!(graph.data.unwrap().nodes.len(), 0);
    }

    #[tokio::test]
    async fn test_add_node() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        let node_data = json!({
            "label": "Test Node",
            "color": "#ff6b6b",
            "size": 25.0
        });

        let response = server.post("/api/nodes").json(&node_data).await;
        response.assert_status_ok();

        let result: ApiResponse<Node> = response.json();
        assert!(result.success);
        let node = result.data.unwrap();
        assert_eq!(node.label, "Test Node");
        assert_eq!(node.color, Some("#ff6b6b".to_string()));
        assert_eq!(node.size, Some(25.0));
    }

    #[tokio::test]
    async fn test_add_edge() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Add two nodes first
        let node1_data = json!({"label": "Node 1"});
        let response1 = server.post("/api/nodes").json(&node1_data).await;
        let node1: ApiResponse<Node> = response1.json();
        let node1_id = node1.data.unwrap().id;

        let node2_data = json!({"label": "Node 2"});
        let response2 = server.post("/api/nodes").json(&node2_data).await;
        let node2: ApiResponse<Node> = response2.json();
        let node2_id = node2.data.unwrap().id;

        // Add edge between them
        let edge_data = json!({
            "source": node1_id,
            "target": node2_id,
            "label": "connects",
            "weight": 0.8
        });

        let response = server.post("/api/edges").json(&edge_data).await;
        response.assert_status_ok();

        let result: ApiResponse<Edge> = response.json();
        assert!(result.success);
        let edge = result.data.unwrap();
        assert_eq!(edge.source, node1_id);
        assert_eq!(edge.target, node2_id);
        assert_eq!(edge.label, Some("connects".to_string()));
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("persistence_test.json");

        // Create first app instance and add data
        {
            let graph_state = Arc::new(RwLock::new(GraphState::new(save_path.clone())));
            let app = Router::new()
                .route("/api/nodes", post(add_node))
                .route("/api/graph", get(get_graph))
                .with_state(graph_state);
            
            let server = TestServer::new(app).unwrap();

            let node_data = json!({
                "id": "test-node-1",
                "label": "Persistent Node",
                "color": "#4ecdc4"
            });

            let response = server.post("/api/nodes").json(&node_data).await;
            response.assert_status_ok();
        }

        // Verify file was created
        assert!(save_path.exists(), "Save file should exist");

        // Create second app instance (simulating restart)
        {
            let graph_state = Arc::new(RwLock::new(GraphState::new(save_path.clone())));
            let app = Router::new()
                .route("/api/graph", get(get_graph))
                .with_state(graph_state);
            
            let server = TestServer::new(app).unwrap();

            let response = server.get("/api/graph").await;
            response.assert_status_ok();

            let graph: ApiResponse<Graph> = response.json();
            assert!(graph.success);
            let data = graph.data.unwrap();
            assert_eq!(data.nodes.len(), 1);
            assert!(data.nodes.contains_key("test-node-1"));
            
            let node = &data.nodes["test-node-1"];
            assert_eq!(node.label, "Persistent Node");
            assert_eq!(node.color, Some("#4ecdc4".to_string()));
        }
    }

    #[tokio::test]
    async fn test_graph_layout_positions() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Add multiple nodes to test layout
        for i in 1..=5 {
            let node_data = json!({
                "label": format!("Node {}", i),
                "size": 20.0 + (i as f64 * 2.0)
            });
            server.post("/api/nodes").json(&node_data).await;
        }

        let response = server.get("/api/graph").await;
        response.assert_status_ok();

        let graph: ApiResponse<Graph> = response.json();
        let data = graph.data.unwrap();
        assert_eq!(data.nodes.len(), 5);

        // Verify all nodes have different sizes
        let mut sizes: Vec<_> = data.nodes.values()
            .map(|n| n.size.unwrap_or(20.0))
            .collect();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        for i in 0..sizes.len()-1 {
            assert!(sizes[i] < sizes[i+1], "Node sizes should be increasing");
        }
    }

    #[tokio::test]
    async fn test_clear_graph() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Add some data
        let node_data = json!({"label": "Test Node"});
        server.post("/api/nodes").json(&node_data).await;

        // Verify data exists
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        assert_eq!(graph.data.unwrap().nodes.len(), 1);

        // Clear graph
        let response = server.post("/api/clear").await;
        response.assert_status_ok();

        // Verify graph is empty
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        assert_eq!(graph.data.unwrap().nodes.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_node_cascades_edges() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Add two nodes
        let node1_data = json!({"id": "node1", "label": "Node 1"});
        let node2_data = json!({"id": "node2", "label": "Node 2"});
        
        server.post("/api/nodes").json(&node1_data).await;
        server.post("/api/nodes").json(&node2_data).await;

        // Add edge between them
        let edge_data = json!({
            "source": "node1",
            "target": "node2",
            "label": "connects"
        });
        server.post("/api/edges").json(&edge_data).await;

        // Verify edge exists
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        let data = graph.data.unwrap();
        assert_eq!(data.edges.len(), 1);

        // Remove node1
        let response = server.delete("/api/nodes/node1").await;
        response.assert_status_ok();

        // Verify edge was also removed
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        let data = graph.data.unwrap();
        assert_eq!(data.nodes.len(), 1);
        assert_eq!(data.edges.len(), 0);
    }

    #[tokio::test]
    async fn test_edge_data_structure_consistency() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Add nodes
        let node1_data = json!({"id": "source-node", "label": "Source Node"});
        let node2_data = json!({"id": "target-node", "label": "Target Node"});
        
        server.post("/api/nodes").json(&node1_data).await;
        server.post("/api/nodes").json(&node2_data).await;

        // Add edge
        let edge_data = json!({
            "source": "source-node",
            "target": "target-node",
            "label": "test-edge",
            "weight": 0.75
        });
        let response = server.post("/api/edges").json(&edge_data).await;
        response.assert_status_ok();
        
        let edge_result: ApiResponse<Edge> = response.json();
        let edge = edge_result.data.unwrap();

        // Verify edge structure
        assert_eq!(edge.source, "source-node");
        assert_eq!(edge.target, "target-node");
        assert_eq!(edge.label, Some("test-edge".to_string()));
        assert_eq!(edge.weight, Some(0.75));

        // Get graph and verify consistency
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        let data = graph.data.unwrap();
        
        assert_eq!(data.nodes.len(), 2);
        assert_eq!(data.edges.len(), 1);
        
        // Edge should reference existing nodes
        let stored_edge = data.edges.values().next().unwrap();
        assert!(data.nodes.contains_key(&stored_edge.source));
        assert!(data.nodes.contains_key(&stored_edge.target));
    }

    #[tokio::test]
    async fn test_multiple_edges_from_same_node() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Create hub node and multiple target nodes
        let hub_data = json!({"id": "hub", "label": "Hub Node"});
        server.post("/api/nodes").json(&hub_data).await;

        for i in 1..=3 {
            let node_data = json!({"id": format!("target{}", i), "label": format!("Target {}", i)});
            server.post("/api/nodes").json(&node_data).await;
        }

        // Add edges from hub to all targets
        for i in 1..=3 {
            let edge_data = json!({
                "source": "hub",
                "target": format!("target{}", i),
                "label": format!("edge{}", i)
            });
            server.post("/api/edges").json(&edge_data).await;
        }

        // Verify all edges exist and are distinct
        let response = server.get("/api/graph").await;
        let graph: ApiResponse<Graph> = response.json();
        let data = graph.data.unwrap();
        
        assert_eq!(data.nodes.len(), 4); // hub + 3 targets
        assert_eq!(data.edges.len(), 3);

        // All edges should have hub as source
        for edge in data.edges.values() {
            assert_eq!(edge.source, "hub");
            assert!(edge.target.starts_with("target"));
        }

        // All targets should be different
        let targets: std::collections::HashSet<_> = data.edges.values().map(|e| &e.target).collect();
        assert_eq!(targets.len(), 3);
    }

    #[tokio::test]
    async fn test_edge_validation() {
        let (app, _temp_dir) = create_test_app();
        let server = TestServer::new(app).unwrap();

        // Try to add edge without nodes - should fail
        let edge_data = json!({
            "source": "nonexistent1",
            "target": "nonexistent2",
            "label": "invalid"
        });
        let response = server.post("/api/edges").json(&edge_data).await;
        response.assert_status_ok();
        
        let result: ApiResponse<Edge> = response.json();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Source node"));

        // Add one node
        let node_data = json!({"id": "valid-node", "label": "Valid Node"});
        server.post("/api/nodes").json(&node_data).await;

        // Try edge with valid source but invalid target
        let edge_data = json!({
            "source": "valid-node",
            "target": "nonexistent",
            "label": "half-valid"
        });
        let response = server.post("/api/edges").json(&edge_data).await;
        let result: ApiResponse<Edge> = response.json();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Target node"));
    }
}