// Basic D3.js graph renderer - TypeScript implementation
// Focus: Get edges to connect properly to nodes

interface GraphNode {
    id: string;
    label: string;
    color?: string;
    size?: number;
    x?: number;
    y?: number;
    vx?: number;
    vy?: number;
}

interface GraphEdge {
    id: string;
    source: string | GraphNode;
    target: string | GraphNode;
    label?: string;
    weight?: number;
    color?: string;
}

interface GraphData {
    nodes: GraphNode[];
    links: GraphEdge[];
}

declare const d3: any;

class BasicGraphRenderer {
    private svg: any;
    private simulation: any;
    private width: number;
    private height: number;
    private container: any;

    constructor(containerId: string, width: number = 800, height: number = 600) {
        this.width = width;
        this.height = height;
        
        // Create SVG
        this.svg = d3.select(containerId)
            .append('svg')
            .attr('width', width)
            .attr('height', height)
            .style('border', '1px solid #ccc');
        
        // Create container for graph elements
        this.container = this.svg.append('g').attr('class', 'graph-container');
        
        // Initialize simulation
        this.initSimulation();
    }

    private initSimulation(): void {
        this.simulation = d3.forceSimulation()
            .force('link', d3.forceLink()
                .id((d: GraphNode) => {
                    console.log('ID accessor called for:', d.id);
                    return d.id;
                })
                .distance(100)
                .strength(0.5))
            .force('charge', d3.forceManyBody().strength(-300))
            .force('center', d3.forceCenter(this.width / 2, this.height / 2))
            .force('collision', d3.forceCollide().radius(30))
            .alphaDecay(0.02)
            .velocityDecay(0.8);
    }

    public render(data: GraphData): void {
        console.log('Rendering graph with:', data.nodes.length, 'nodes and', data.links.length, 'links');
        
        // Clear previous render
        this.container.selectAll('*').remove();
        
        // Create copies of data for D3 (D3 mutates the data)
        const nodes = data.nodes.map(d => ({ ...d }));
        const links = data.links.map(d => ({ ...d }));
        
        console.log('Initial links:', links.map(l => ({ 
            id: l.id, 
            source: l.source, 
            target: l.target 
        })));

        // Update simulation
        this.simulation.nodes(nodes);
        this.simulation.force('link').links(links);

        // Create visual elements
        this.createLinks(links);
        this.createNodes(nodes);
        
        // Set up tick handler
        this.setupTickHandler(nodes, links);
        
        // Start simulation
        this.simulation.alpha(1).restart();
        
        // Log links after D3 processes them
        setTimeout(() => {
            console.log('Links after D3 processing:', links.map(l => ({
                id: l.id,
                source: typeof l.source === 'string' ? l.source : l.source.id,
                target: typeof l.target === 'string' ? l.target : l.target.id,
                sourceType: typeof l.source,
                targetType: typeof l.target
            })));
        }, 100);
    }

    private createLinks(links: GraphEdge[]): void {
        const linkGroup = this.container.append('g').attr('class', 'links');
        
        linkGroup.selectAll('line')
            .data(links)
            .enter()
            .append('line')
            .attr('class', 'link')
            .style('stroke', d => d.color || '#999')
            .style('stroke-width', 2)
            .style('stroke-opacity', 0.6);
    }

    private createNodes(nodes: GraphNode[]): void {
        const nodeGroup = this.container.append('g').attr('class', 'nodes');
        
        const nodeElements = nodeGroup.selectAll('circle')
            .data(nodes)
            .enter()
            .append('circle')
            .attr('class', 'node')
            .attr('r', d => d.size || 20)
            .style('fill', d => d.color || '#69b3a2')
            .style('stroke', '#333')
            .style('stroke-width', 2)
            .style('cursor', 'pointer');

        // Add labels
        const labelGroup = this.container.append('g').attr('class', 'labels');
        
        labelGroup.selectAll('text')
            .data(nodes)
            .enter()
            .append('text')
            .attr('class', 'node-label')
            .text(d => d.label)
            .style('text-anchor', 'middle')
            .style('dominant-baseline', 'central')
            .style('font-size', '12px')
            .style('font-weight', 'bold')
            .style('pointer-events', 'none');
    }

    private setupTickHandler(nodes: GraphNode[], links: GraphEdge[]): void {
        this.simulation.on('tick', () => {
            // Update link positions
            this.container.selectAll('.link')
                .attr('x1', (d: GraphEdge) => this.getLinkX1(d, nodes))
                .attr('y1', (d: GraphEdge) => this.getLinkY1(d, nodes))
                .attr('x2', (d: GraphEdge) => this.getLinkX2(d, nodes))
                .attr('y2', (d: GraphEdge) => this.getLinkY2(d, nodes));

            // Update node positions
            this.container.selectAll('.node')
                .attr('cx', (d: GraphNode) => d.x!)
                .attr('cy', (d: GraphNode) => d.y!);

            // Update label positions
            this.container.selectAll('.node-label')
                .attr('x', (d: GraphNode) => d.x!)
                .attr('y', (d: GraphNode) => d.y! + 4);
        });
    }

    private getLinkX1(d: GraphEdge, nodes: GraphNode[]): number {
        if (typeof d.source === 'string') {
            console.warn('Source is still string:', d.source);
            const sourceNode = nodes.find(n => n.id === d.source);
            return sourceNode?.x ?? this.width / 2;
        }
        return d.source.x ?? this.width / 2;
    }

    private getLinkY1(d: GraphEdge, nodes: GraphNode[]): number {
        if (typeof d.source === 'string') {
            const sourceNode = nodes.find(n => n.id === d.source);
            return sourceNode?.y ?? this.height / 2;
        }
        return d.source.y ?? this.height / 2;
    }

    private getLinkX2(d: GraphEdge, nodes: GraphNode[]): number {
        if (typeof d.target === 'string') {
            console.warn('Target is still string:', d.target);
            const targetNode = nodes.find(n => n.id === d.target);
            return targetNode?.x ?? this.width / 2;
        }
        return d.target.x ?? this.width / 2;
    }

    private getLinkY2(d: GraphEdge, nodes: GraphNode[]): number {
        if (typeof d.target === 'string') {
            const targetNode = nodes.find(n => n.id === d.target);
            return targetNode?.y ?? this.height / 2;
        }
        return d.target.y ?? this.height / 2;
    }

    public destroy(): void {
        if (this.simulation) {
            this.simulation.stop();
        }
        if (this.svg) {
            this.svg.remove();
        }
    }
}

// Export for use in other files
(window as any).BasicGraphRenderer = BasicGraphRenderer;