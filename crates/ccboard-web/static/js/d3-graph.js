/**
 * D3.js force-directed graph visualization for task dependencies
 *
 * Renders an interactive graph with:
 * - Force simulation for natural layout
 * - Color-coded nodes by status (Complete/InProgress/Future)
 * - Directed edges with arrows
 * - Drag interaction
 * - Zoom/pan support
 */

window.renderTaskGraph = function(nodes, edges) {
    const width = 1200;
    const height = 600;

    // Clear previous graph
    d3.select("#d3-graph").selectAll("*").remove();

    const svg = d3.select("#d3-graph")
        .append("svg")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr("viewBox", `0 0 ${width} ${height}`)
        .attr("preserveAspectRatio", "xMidYMid meet");

    // Add zoom behavior
    const g = svg.append("g");

    const zoom = d3.zoom()
        .scaleExtent([0.1, 4])
        .on("zoom", (event) => {
            g.attr("transform", event.transform);
        });

    svg.call(zoom);

    // Define arrow markers for edges
    svg.append("defs").selectAll("marker")
        .data(["arrow"])
        .enter().append("marker")
        .attr("id", d => d)
        .attr("viewBox", "0 -5 10 10")
        .attr("refX", 25)
        .attr("refY", 0)
        .attr("markerWidth", 6)
        .attr("markerHeight", 6)
        .attr("orient", "auto")
        .append("path")
        .attr("d", "M0,-5L10,0L0,5")
        .attr("fill", "#999");

    // Force simulation
    const simulation = d3.forceSimulation(nodes)
        .force("link", d3.forceLink(edges)
            .id(d => d.id)
            .distance(150))
        .force("charge", d3.forceManyBody()
            .strength(-400))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .force("collision", d3.forceCollide()
            .radius(60));

    // Render edges
    const link = g.append("g")
        .attr("class", "links")
        .selectAll("line")
        .data(edges)
        .enter().append("line")
        .attr("stroke", "#666")
        .attr("stroke-width", 2)
        .attr("marker-end", "url(#arrow)");

    // Render nodes
    const node = g.append("g")
        .attr("class", "nodes")
        .selectAll("g")
        .data(nodes)
        .enter().append("g")
        .call(d3.drag()
            .on("start", dragStarted)
            .on("drag", dragged)
            .on("end", dragEnded));

    // Node circles
    node.append("circle")
        .attr("r", 20)
        .attr("fill", d => statusColor(d.status))
        .attr("stroke", "#fff")
        .attr("stroke-width", 2);

    // Node labels
    node.append("text")
        .text(d => d.id)
        .attr("dx", 25)
        .attr("dy", 5)
        .attr("font-size", "12px")
        .attr("font-weight", "bold")
        .attr("fill", "#fff");

    // Node title (task name) - appears on hover
    node.append("title")
        .text(d => `${d.id}: ${d.label}\nPhase: ${d.phase}\nStatus: ${d.status}\nDuration: ${d.duration || 'N/A'}`);

    // Update positions on each tick
    simulation.on("tick", () => {
        link
            .attr("x1", d => d.source.x)
            .attr("y1", d => d.source.y)
            .attr("x2", d => d.target.x)
            .attr("y2", d => d.target.y);

        node
            .attr("transform", d => `translate(${d.x},${d.y})`);
    });

    /**
     * Map task status to color
     */
    function statusColor(status) {
        switch(status.toLowerCase()) {
            case "complete":
            case "completed":
                return "#4CAF50"; // Green
            case "inprogress":
            case "in-progress":
            case "in_progress":
                return "#FFC107"; // Yellow
            case "future":
            default:
                return "#9E9E9E"; // Grey
        }
    }

    /**
     * Drag event handlers
     */
    function dragStarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragEnded(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }
};
