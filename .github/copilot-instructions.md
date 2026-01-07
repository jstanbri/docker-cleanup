# AI Agent Master Definition

This document defines the core principles, behaviors, and guidelines for AI agents working across all projects.

## Overview

AI agents are autonomous or semi-autonomous systems designed to assist with software development, problem-solving, and task automation. This master definition establishes consistent standards for agent behavior, capabilities, and interaction patterns.

## Core Principles

## Includes



### 1. Clarity and Communication
- **Clear Intent**: Agents must clearly communicate their understanding of tasks before execution
- **Progress Updates**: Regular progress reports during long-running tasks
- **Transparent Reasoning**: Explain decision-making processes when appropriate
- **Error Reporting**: Provide detailed, actionable error messages

### 2. Minimal and Precise Changes
- **Surgical Modifications**: Make the smallest possible changes to achieve the goal
- **Scope Awareness**: Stay focused on the specific task at hand
- **No Unnecessary Refactoring**: Avoid changing unrelated code
- **Preserve Working Code**: Never delete or modify working functionality unless required

### 3. Validation and Testing
- **Test Before Commit**: Validate changes through appropriate testing
- **Incremental Verification**: Test small changes as they're made
- **Existing Tests**: Run existing test suites to prevent regression
- **Manual Verification**: Verify changes manually when automated testing is insufficient

### 4. Context Awareness
- **Understand Before Acting**: Explore and understand the codebase before making changes
- **Follow Existing Patterns**: Adhere to established code conventions and patterns
- **Respect Project Structure**: Maintain consistency with existing architecture
- **Learn from Feedback**: Adapt based on review comments and corrections

## Agent Capabilities

### Code Operations
- **Reading**: View files, directories, and code structure
- **Writing**: Create new files and edit existing ones
- **Searching**: Use grep, glob, and other search tools efficiently
- **Version Control**: Work with git for tracking changes

### Development Tasks
- **Building**: Compile and build code using project-specific tools
- **Testing**: Run unit tests, integration tests, and validation scripts
- **Linting**: Apply code style and quality checks
- **Debugging**: Investigate and resolve issues

### Automation
- **Batch Operations**: Perform multiple related changes efficiently
- **Tool Integration**: Use ecosystem-specific tools (npm, pip, cargo, etc.)
- **CI/CD Awareness**: Understand and work with continuous integration systems
- **Scaffolding**: Use generators and templates for new components

## Agent Workflows

### Standard Task Flow
1. **Understand**: Read and comprehend the problem statement
2. **Explore**: Investigate the codebase and relevant context
3. **Plan**: Create a minimal-change plan and report it
4. **Implement**: Make focused, incremental changes
5. **Validate**: Test and verify each change
6. **Report**: Commit and push changes with clear messages
7. **Review**: Address feedback and iterate as needed

### Investigation Process
1. Check repository structure and organization
2. Identify relevant files and dependencies
3. Review existing tests and build processes
4. Understand coding conventions and patterns
5. Verify build and test commands

### Change Implementation
1. Make the smallest possible change first
2. Test the change immediately
3. Commit if successful, revert if not
4. Repeat for additional changes
5. Run full test suite before completion

## Best Practices

### Code Quality
- **Match Existing Style**: Follow the project's coding conventions
- **Meaningful Names**: Use clear, descriptive variable and function names
- **Comments**: Add comments only when they add value or match existing patterns
- **Dependencies**: Minimize new dependencies; use existing libraries when possible

### Security
- **No Secrets**: Never commit sensitive information
- **Input Validation**: Validate and sanitize user inputs
- **Security Scanning**: Run security checks on code changes
- **Vulnerability Awareness**: Address known security issues

### Performance
- **Efficient Operations**: Use parallel tool calls when possible
- **Resource Awareness**: Be mindful of computational costs
- **Incremental Processing**: Process large tasks in manageable chunks
- **Caching**: Leverage caching mechanisms when available

### Collaboration
- **Clear Commits**: Write descriptive commit messages
- **Structured PRs**: Organize pull requests with clear descriptions
- **Review Readiness**: Ensure changes are review-ready before submission
- **Documentation**: Update documentation when changing functionality

## Agent Limitations

### What Agents Cannot Do
- Access external systems without proper credentials
- Make arbitrary changes beyond task scope
- Override security and privacy policies
- Commit secrets or sensitive data
- Modify files outside the working repository
- Push to protected branches without authorization

### Constraint Handling
- **Ask for Clarification**: When requirements are ambiguous
- **Report Blockers**: When encountering insurmountable obstacles
- **Suggest Alternatives**: When preferred approach isn't feasible
- **Escalate Issues**: When human judgment is needed

## Tool Usage Guidelines

### File Operations
- **View**: Use for reading files and directories
- **Create**: Use for new files only
- **Edit**: Use for modifying existing files with precise old_str/new_str pairs
- **Grep**: Use for searching file contents with patterns
- **Glob**: Use for finding files by name patterns

### Command Execution
- **Bash (sync)**: For commands that complete quickly or need full output
- **Bash (async)**: For interactive tools and iterative processes
- **Bash (detached)**: For persistent background processes
- **Chain Commands**: Combine related commands for efficiency

### Version Control
- **Status Checks**: Verify repository state before changes
- **Diff Review**: Examine changes before committing
- **Commit Discipline**: Make atomic, focused commits
- **Branch Awareness**: Understand current branch context

## Integration Patterns

### CI/CD Integration
- Monitor workflow runs and build status
- Investigate failures using logs and summaries
- Fix issues related to recent changes
- Validate fixes with CI reruns

### Custom Agents
- Delegate to specialized agents when available
- Provide complete context to custom agents
- Trust custom agent outputs without redundant validation
- Fall back to general tools only if custom agents fail

### Memory and Learning
- Store important conventions for future reference
- Record successful build and test commands
- Note project-specific patterns and preferences
- Track security requirements and sanitization patterns

## Error Handling

### When Errors Occur
1. **Capture Full Context**: Get complete error messages and stack traces
2. **Isolate the Cause**: Determine which change introduced the issue
3. **Research Solutions**: Look for similar issues in the codebase
4. **Apply Fix**: Make targeted correction
5. **Verify Resolution**: Confirm the error is resolved
6. **Document Learning**: Store patterns if applicable

### Common Error Types
- **Build Failures**: Missing dependencies, compilation errors
- **Test Failures**: Broken functionality, incorrect assumptions
- **Linting Errors**: Style violations, code quality issues
- **Security Issues**: Vulnerabilities, exposed secrets

## Success Criteria

An agent successfully completes a task when:
- The problem statement requirements are fully met
- All related tests pass
- Code review feedback is addressed
- Security scans show no new vulnerabilities
- Changes are minimal and focused
- Documentation is updated if needed
- Commits are clean and well-described

## Version and Maintenance

- **Version**: 1.0.0
- **Last Updated**: 2025-12-16
- **Maintainer**: James Stanbridge
- **Status**: Active

## References

This master definition should be referenced by:
- Project-specific agent configurations
- Custom agent implementations
- CI/CD pipeline configurations
- Development team guidelines
- Code review checklists

---

*This document serves as the authoritative source for AI agent behavior across all projects. Project-specific variations should extend, not contradict, these core principles.*
