# Agents

This project is developed with the assistance of AI coding agents.

## Claude Code

[Claude Code](https://docs.anthropic.com/en/docs/claude-code) is used as the primary AI-assisted development tool for this project.

### What Claude Code handles

- **Code generation** - Writing new features, modules, and boilerplate
- **Testing** - Generating and maintaining unit tests
- **CI/CD** - Setting up and maintaining GitHub Actions workflows
- **Documentation** - Creating and updating project documentation
- **Code review** - Identifying bugs, suggesting improvements, and enforcing patterns

### Development workflow

1. Describe the feature or change needed in natural language
2. Claude Code explores the codebase to understand existing patterns
3. Changes are implemented following project conventions
4. Tests are written alongside the implementation
5. All changes are reviewed before committing

### Guidelines for AI-assisted contributions

- Always review generated code before merging
- Verify tests pass on all target platforms (Windows, Linux, macOS)
- Keep the human in the loop for architectural decisions
- AI-generated commits are co-authored with `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`
