# README Milestone Section

Use this section in all Univrs ecosystem READMEs:

---

## 📊 Development Status

```
Phase 1  ████████████████████  Parser + Lexer     ✅ 150 tests
Phase 2a ████████████████████  HIR v0.4.0         ✅ 466 tests
Phase 2b ████████████████████  VUDO VM            ✅ 402 tests
Phase 2c ████████████████████  Spirit Runtime     ✅ 50 tests
Phase 3  ████████████████████  DOL → WASM         ✅ dol 0.8.2 (direct backend, no MLIR)
Phase 4a ████████████████████  Hyphal Network     ✅ 38 tests
Phase 4b ████████░░░░░░░░░░░░  ENR Economics      🔄 Active
```

**DOL Tests:** 2,512 (dol 0.8.2) | **DOL Specs:** 3,094 lines

[View Full Milestones →](https://github.com/univrs/univrs-dol/wiki/Milestones)

---

## Quick Copy Badges (for GitHub)

### SVG Badge (Phase Status)
```html
<img src="https://img.shields.io/badge/Phase-4b%20ENR-00ff88?style=flat-square&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PGNpcmNsZSBjeD0iMTIiIGN5PSIxMiIgcj0iOCIgZmlsbD0iIzAwZmY4OCIvPjwvc3ZnPg==" alt="Phase 4b">
```

### Markdown Badge
```markdown
![Phase](https://img.shields.io/badge/Phase-4b%20ENR-00ff88?style=flat-square)
![DOL Tests](https://img.shields.io/badge/DOL%20Tests-2512-brightgreen?style=flat-square)
![DOL](https://img.shields.io/badge/DOL-3094%20lines-blue?style=flat-square)
```

### Shields.io Dynamic (if you have a JSON endpoint)
```markdown
![Status](https://img.shields.io/endpoint?url=https://univrs.io/api/status.json)
```

---

## Ecosystem Links Section

```markdown
## 🌐 Ecosystem

| Site | Purpose |
|------|---------|
| [univrs.io](https://univrs.io) | Main portal |
| [vudo.univrs.io](https://vudo.univrs.io) | VM documentation |
| [learn.univrs.io](https://learn.univrs.io) | Tutorials |

### Repositories

| Repo | Status |
|------|--------|
| [dol](https://github.com/univrs/dol) | ![](https://img.shields.io/badge/2512-tests-00ff88) |
| [univrs-vudo](https://github.com/univrs/vudo) | ![](https://img.shields.io/badge/402-tests-00ff88) |
| [univrs-enr](https://github.com/univrs/enr) | ![](https://img.shields.io/badge/active-yellow) |
| [univrs-network](https://github.com/univrs/network) | ![](https://img.shields.io/badge/pending-gray) |
```

---

## Wiki Sidebar Section

For GitHub Wiki `_Sidebar.md`:

```markdown
### Navigation

- [[Home]]
- [[Milestones]]
- [[Architecture]]

### Phases

- [[Phase 1 - Parser|phase-1-parser]]
- [[Phase 2 - HIR & VM|phase-2-hir-vm]]
- [[Phase 3 - WASM|phase-3-wasm]]
- [[Phase 4 - ENR|phase-4-enr]] 🔄

### Reference

- [[DOL Specification|dol-spec]]
- [[ENR Architecture|enr-architecture]]
- [[API Reference|api-reference]]
```

---

## Claude Code Integration

To add the MilestoneTracker component to your sites:

```bash
# Copy component to univrs.io
cp MilestoneTracker.tsx ~/repos/univrs.io/src/components/

# Copy component to vudo.univrs.io  
cp MilestoneTracker.tsx ~/repos/vudo.univrs.io/src/components/

# Import in App.tsx
# Add: import { MilestoneTracker } from './components/MilestoneTracker';
# Use: <MilestoneTracker />
```

Or ask Claude Code:
```
Add the MilestoneTracker component from /outputs/MilestoneTracker.tsx 
to the univrs.io site, matching the existing bioluminescent aesthetic.
```
