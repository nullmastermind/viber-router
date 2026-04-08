## 1. Fix Dialog Card Sizing

- [x] 1.1 In `src/pages/PublicUsagePage.vue` line 273, replace `style="min-width: 480px; max-width: 95vw"` with `style="width: 95vw; max-width: 480px"` on the `q-card` element
- [x] 1.2 On line 278, add `style="overflow-x: auto"` to the `q-card-section` that wraps the `q-markup-table` ← (verify: dialog does not overflow viewport on a 375px-wide screen; table section scrolls horizontally when columns exceed card width)

## 2. Verify

- [x] 2.1 Run `just check` and confirm zero lint and type errors
