# Claude Code Instructions

## Native Hooks Mode (Context Engine)

This project uses Claude Code native hooks for context management.

**Automatic behaviors:**
- SessionStart: Fresh context injected on /clear, resume, or new session
- PreCompact: State saved before compaction
- Stop: Progress tracked when you finish responding

**Commands:**
- `.agent/commands.sh recall failures` - See what NOT to do
- `.agent/commands.sh success <id> <msg>` - Mark feature complete
- `.agent/commands.sh failure <id> <msg>` - Record failure
- `.agent/commands.sh status` - Check progress

**After /clear or /compact:** Context is automatically restored. Just continue working.

## Workflow

1. Check current task: `.agent/commands.sh status`
2. Check failures: `.agent/commands.sh recall failures`
3. Implement the feature
4. Run tests
5. Mark complete: `.agent/commands.sh success <id> "what you did"`
6. Commit: `git add -A && git commit -m "completed <id>"`

The context engine handles the rest automatically.
