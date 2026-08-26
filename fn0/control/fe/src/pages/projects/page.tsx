import type { Props } from "./.props";

export default function ProjectsPage(props: Props) {
    return (
        <div style={{ maxWidth: 720, margin: "2rem auto", fontFamily: "system-ui" }}>
            <h1>Projects</h1>
            <p>
                Signed in as <strong>{props.githubLogin}</strong>
            </p>
            {props.projects.length === 0 ? (
                <p>No projects yet.</p>
            ) : (
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                    <thead>
                        <tr>
                            <th style={cell}>Name</th>
                            <th style={cell}>Project ID</th>
                            <th style={cell}></th>
                        </tr>
                    </thead>
                    <tbody>
                        {props.projects.map((project) => (
                            <tr key={project.projectId}>
                                <td style={cell}>{project.name}</td>
                                <td style={{ ...cell, fontFamily: "monospace" }}>
                                    {project.projectId}
                                </td>
                                <td style={cell}>
                                    <a
                                        href={`/projects/${encodeURIComponent(project.projectId)}/logs`}
                                    >
                                        Logs
                                    </a>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    );
}

const cell: React.CSSProperties = {
    padding: 8,
    borderBottom: "1px solid #eee",
    textAlign: "left",
};
