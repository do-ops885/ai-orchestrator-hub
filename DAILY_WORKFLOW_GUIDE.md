# 🚫 Daily Unwrap() Prevention Workflow - Quick Reference

## ⚡ **BEFORE YOU START CODING** (30 seconds)

```bash
# Quick security health check
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls
```

**Expected output**: `✅ SUCCESS: No unwrap() calls found in production code`

---

## 💻 **DURING DEVELOPMENT** (Use these patterns)

### ✅ **SAFE PATTERNS** (Always use these)
```rust
// Safe default values
let config = parse_config().unwrap_or_default();
let value = result.unwrap_or_else(|| handle_error());

// Proper error propagation  
let data = load_data()?;  // Use ? operator
let result = match parse_input() {
    Ok(data) => process(data),
    Err(e) => return Err(e.into()),
};

// Safe optional handling
if let Some(value) = optional_data {
    process_value(value);
}
```

### ❌ **FORBIDDEN PATTERNS** (Never use these)
```rust
let value = result.unwrap();           // ❌ Can panic!
let value = option.expect("message");  // ❌ Can panic!
```

---

## 🔍 **BEFORE EVERY COMMIT** (1 minute)

```bash
# 1. Security check (MANDATORY)
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls

# 2. Lint with unwrap() detection
cd backend && cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used

# 3. Run tests
cargo test

# 4. Commit (pre-commit hook will auto-scan)
git add .
git commit -m "Your commit message"
```

---

## 🚨 **IF UNWRAP() DETECTED** (Emergency fix)

```bash
# 1. See what's wrong
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls

# 2. Get help
cat docs/UNWRAP_ALTERNATIVES.md

# 3. Apply fixes, then re-check
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls
```

---

## 🎯 **VS Code Integration** (One-time setup)

1. Copy the settings we created: `.vscode/settings.json` ✅ **Already done!**
2. Install Rust Analyzer extension
3. You'll get **real-time warnings** for unwrap() calls

### **Quick VS Code Tasks**
- `Ctrl+Shift+P` → "Tasks: Run Task" → "🚫 Check Unwrap Calls"
- `Ctrl+Shift+P` → "Tasks: Run Task" → "🛡️ Security Clippy Check"

---

## 📋 **Code Review Checklist**

When reviewing agent code:
- [ ] No `.unwrap()` calls in production code
- [ ] No `.expect()` calls in production code  
- [ ] Proper error handling with `Result<T, E>`
- [ ] Error scenarios are tested
- [ ] Default values provided where appropriate

---

## 🆘 **Quick Help Commands**

```bash
# Check current status
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls

# Check recent commits  
./scripts/unwrap-prevention-monitor.sh check_recent_commits "24 hours ago"

# Get full help
./scripts/unwrap-prevention-monitor.sh --help

# View alternatives guide
cat docs/UNWRAP_ALTERNATIVES.md
```

---

## 🎉 **Success Indicators**

✅ **You're doing it right when:**
- Pre-commit hook blocks unwrap() commits
- Clippy shows no unwrap() warnings
- CI/CD checks pass automatically
- No production panics or crashes

❌ **Red flags:**
- Getting unwrap() alerts
- Pre-commit hook failures
- CI/CD blocking your PRs

---

## 💡 **Pro Tips**

1. **Set up auto-save** in VS Code to get instant feedback
2. **Use `?` operator** liberally for error propagation
3. **Always provide defaults** with `unwrap_or()`
4. **Test error paths** not just success cases
5. **Read the alternatives guide** - it has examples for every scenario

---

**Remember: Every unwrap() is a potential panic waiting to happen! 🚫⚡**

**Our goal: ZERO production panics through proper error handling! 🛡️✨**