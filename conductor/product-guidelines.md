# AI Video Editor (CLI) - Product Guidelines

## Prose Style & Tone
- **Technical & Precise:** Use accurate terminology. Communication should be clear, unambiguous, and focused on technical details that power users need.
- **Concise:** Avoid unnecessary fluff in both CLI output and documentation.

## User Experience (CLI)
- **Unix Philosophy:** Design commands to be modular and single-purpose. Each tool should do one thing well and work together with others.
- **Pipeable:** Support standard input/output streams to allow integration into larger automation pipelines.
- **Logic-First:** Prioritize the underlying processing engine's robustness and flexibility over visual flair.

## Documentation Standards
- **Example-Driven:** Always lead with practical, copy-pasteable examples for common tasks (e.g., "How to trim silences from a 10-minute video").
- **Clear Hierarchy:** Organize information logically, starting with the most frequent use cases.
- **Technical Depth:** Provide detailed explanations of the AI logic and configuration parameters where necessary.

## Error Handling & Reliability
- **Action-Oriented Errors:** Error messages must not only state what went wrong but also provide a clear path to resolution.
- **Predictable Exit Codes:** Use standard exit codes to facilitate scripting and automation.
- **Validation-First:** Validate inputs and configurations early to prevent unnecessary processing cycles.
