# CodeRabbit Review Before Opening a PR

Before creating a pull request for any branch, run the CodeRabbit skill to catch bugs, security issues, and anti-pattern violations before they land in review.

**When to apply:** Any time you are about to open a PR, have finished implementing a feature/fix, or the user indicates a branch is PR-ready.

**How to run:**

Always review committed changes on the branch compared against `develop`, so the full diff matches exactly what the PR will show:

```bash
coderabbit review --agent -t committed --base develop
```

Do not use `-t uncommitted` or omit `--base` — those scope the review to local working-tree state or the default base, which can miss committed changes or include noise from unrelated edits.

**What to do with findings:**
- **Critical / Warning** — fix before opening the PR. Re-run the review after fixing to confirm clean.
- **Info / Minor** — assess whether each is worth fixing. If skipping, note the reason briefly.

Do not open a PR until the review is either clean or all remaining findings are consciously accepted Info-level items.
