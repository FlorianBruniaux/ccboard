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
    console.log("🎯 renderTaskGraph called with:", { nodes, edges });
    console.log("📊 Nodes count:", nodes.length, "Edges count:", edges.length);

    // Check if D3 is loaded
    if (typeof d3 === 'undefined') {
        console.error("❌ D3.js is not loaded!");
        return;
    }
    console.log("✅ D3.js version:", d3.version);

    // Check if target element exists
    const target = document.getElementById("d3-graph");
    if (!target) {
        console.error("❌ #d3-graph element not found!");
        return;
    }
    console.log("✅ Target element found:", target);

    const width = 1200;
    const height = 600;

    // Clear previous graph
    d3.select("#d3-graph").selectAll("*").remove();
    console.log("🧹 Previous graph cleared");

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

    // Force simulation (adjusted for larger nodes with labels)
    const simulation = d3.forceSimulation(nodes)
        .force("link", d3.forceLink(edges)
            .id(d => d.id)
            .distance(200))
        .force("charge", d3.forceManyBody()
            .strength(-500))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .force("collision", d3.forceCollide()
            .radius(100));

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

    // Node circles (bigger to accommodate labels)
    node.append("circle")
        .attr("r", 30)
        .attr("fill", d => statusColor(d.status))
        .attr("stroke", "#fff")
        .attr("stroke-width", 2);

    // Task ID inside circle
    node.append("text")
        .text(d => d.id)
        .attr("text-anchor", "middle")
        .attr("dy", 5)
        .attr("font-size", "11px")
        .attr("font-weight", "bold")
        .attr("fill", "#fff");

    // Task title below circle
    node.append("text")
        .text(d => {
            // Truncate long titles
            const maxLength = 30;
            return d.label.length > maxLength
                ? d.label.substring(0, maxLength) + '...'
                : d.label;
        })
        .attr("text-anchor", "middle")
        .attr("dy", 45)
        .attr("font-size", "12px")
        .attr("fill", "#ccc");

    // Click handler to show tooltip
    node.on("click", function(event, d) {
        event.stopPropagation(); // Prevent background click
        showTooltip(d, event);
    });

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

    /**
     * Show rich HTML tooltip for task node
     */
    function showTooltip(taskData, event) {
        const tooltip = document.getElementById("task-tooltip");
        if (!tooltip) return;

        // Populate tooltip content
        document.getElementById("tooltip-title").textContent =
            `${taskData.id}: ${taskData.label}`;

        const content = document.getElementById("tooltip-content");
        content.innerHTML = formatTooltipContent(taskData);

        // Position tooltip near cursor
        const x = event.pageX + 15;
        const y = event.pageY + 15;

        tooltip.style.left = `${x}px`;
        tooltip.style.top = `${y}px`;
        tooltip.classList.remove("hidden");
    }

    /**
     * Hide tooltip
     */
    function hideTooltip() {
        const tooltip = document.getElementById("task-tooltip");
        if (tooltip) {
            tooltip.classList.add("hidden");
        }
    }

    /**
     * Format tooltip HTML content with sections
     */
    function formatTooltipContent(task) {
        let html = '<div class="tooltip-section">';

        // Basic info: Status + Phase + Duration
        html += `<div class="tooltip-row">
            <span class="tooltip-label">Status:</span>
            <span class="tooltip-value status-${task.status.toLowerCase()}">${task.status}</span>
        </div>`;

        html += `<div class="tooltip-row">
            <span class="tooltip-label">Phase:</span>
            <span class="tooltip-value">${task.phase}</span>
        </div>`;

        if (task.duration) {
            html += `<div class="tooltip-row">
                <span class="tooltip-label">Duration:</span>
                <span class="tooltip-value">${task.duration}</span>
            </div>`;
        }

        html += '</div>';

        // Metadata: Priority + Difficulty
        if (task.priority || task.difficulty) {
            html += '<div class="tooltip-section">';

            if (task.priority) {
                html += `<div class="tooltip-row">
                    <span class="tooltip-label">Priority:</span>
                    <span class="tooltip-value">${task.priority}</span>
                </div>`;
            }

            if (task.difficulty) {
                html += `<div class="tooltip-row">
                    <span class="tooltip-label">Difficulty:</span>
                    <span class="tooltip-value">${task.difficulty}</span>
                </div>`;
            }

            html += '</div>';
        }

        // Technical info: Crate + Issue
        if (task.crateName || task.issue) {
            html += '<div class="tooltip-section">';

            if (task.crateName) {
                html += `<div class="tooltip-row">
                    <span class="tooltip-label">Crate:</span>
                    <span class="tooltip-value"><code>${task.crateName}</code></span>
                </div>`;
            }

            if (task.issue) {
                html += `<div class="tooltip-row">
                    <span class="tooltip-label">Issue:</span>
                    <span class="tooltip-value">
                        <a href="https://github.com/FlorianBruniaux/ccboard/issues/${task.issue}"
                           target="_blank">#${task.issue}</a>
                    </span>
                </div>`;
            }

            html += '</div>';
        }

        // Description (if available)
        if (task.description) {
            html += '<div class="tooltip-section tooltip-description">';
            html += `<div class="tooltip-label">Description:</div>`;

            // Truncate long descriptions
            const maxLength = 300;
            const desc = task.description;
            const truncated = desc.length > maxLength
                ? desc.substring(0, maxLength) + '...'
                : desc;

            html += `<div class="tooltip-value">${escapeHtml(truncated)}</div>`;
            html += '</div>';
        }

        return html;
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Close tooltip on background click
    svg.on("click", function() {
        hideTooltip();
    });

    // Close tooltip on Escape key
    document.addEventListener("keydown", function(e) {
        if (e.key === "Escape") {
            hideTooltip();
        }
    });

    // Close button handler
    document.addEventListener("click", function(e) {
        if (e.target && e.target.id === "tooltip-close") {
            hideTooltip();
        }
    });
};
