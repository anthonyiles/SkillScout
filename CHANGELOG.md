# Changelog

## [0.3.2](https://github.com/anthonyiles/SkillScout/compare/skillscout-v0.3.1...skillscout-v0.3.2) (2026-06-04)


### Bug Fixes

* align E2E setup with working Tauri v2 webdriver examples ([be2c00c](https://github.com/anthonyiles/SkillScout/commit/be2c00c37455059e82085c46185ac76bcb41b6c5))
* bake prod GitHub OAuth client ID into binary ([89fd0f0](https://github.com/anthonyiles/SkillScout/commit/89fd0f03726f75aed652bf9d260e77977595e46c))
* bake prod GitHub OAuth client ID into binary ([#31](https://github.com/anthonyiles/SkillScout/issues/31)) ([a9f351f](https://github.com/anthonyiles/SkillScout/commit/a9f351fff3de86754776fcb468e82615188aa0ce))
* drop Ubuntu E2E, use waitForExist and JS clicks for Windows ([6fdb92c](https://github.com/anthonyiles/SkillScout/commit/6fdb92c9afe50efb4056ef44f7a0a8f25f69cb06))
* enable devtools feature for E2E builds and fix Linux browserName ([def5b66](https://github.com/anthonyiles/SkillScout/commit/def5b66d0debaeb33e22a4634d67c6730fc2c591))
* explicitly navigate to tauri://localhost/ before running E2E tests ([1e30f50](https://github.com/anthonyiles/SkillScout/commit/1e30f50ff6c9bde54a84f3ede4cb06802af6382f))
* pass signing key to E2E tauri build step ([51a5d98](https://github.com/anthonyiles/SkillScout/commit/51a5d98a72349909e13399a88840185830582acc))
* resolve all three E2E test failures ([d436a3a](https://github.com/anthonyiles/SkillScout/commit/d436a3adfe1a7d7a406e929c0d462662efc1c00b))
* resolve E2E CI failures on all platforms ([f55bd33](https://github.com/anthonyiles/SkillScout/commit/f55bd335e1ae68d1e9c4cf253d284f2015a654c5))
* resolve release pipeline — updater routing, signing, and CI configuration ([95553c9](https://github.com/anthonyiles/SkillScout/commit/95553c904b01243b510199ad9389a6ddbb5d28d2))
* switch to ubuntu-24.04 for E2E and maximize window before tests ([11119ec](https://github.com/anthonyiles/SkillScout/commit/11119ec774cb3bc7564b930265ab55415d99026a))
* wait for Tauri frontend to load before running E2E tests ([9f22333](https://github.com/anthonyiles/SkillScout/commit/9f22333fbba70dca07ac55e47fa946dee83560f3))
* wrap latest.json fetch in try-catch to avoid opaque 500 on network error ([f9cac31](https://github.com/anthonyiles/SkillScout/commit/f9cac3197a30acf746f9bbd15262821b76c7147c))

## [0.4.1](https://github.com/anthonyiles/SkillScout/compare/skillscout-v0.4.0...skillscout-v0.4.1) (2026-06-04)


### Bug Fixes

* bake prod GitHub OAuth client ID into binary ([#31](https://github.com/anthonyiles/SkillScout/issues/31)) ([a9f351f](https://github.com/anthonyiles/SkillScout/commit/a9f351fff3de86754776fcb468e82615188aa0ce))

## [0.4.0](https://github.com/anthonyiles/SkillScout/compare/skillscout-v0.3.1...skillscout-v0.4.0) (2026-05-31)


### Features

* add beta tester toggle for channel-based update routing ([c098e21](https://github.com/anthonyiles/SkillScout/commit/c098e21b7b7594aee8fd261c9b14e3e1bd38bccc))
* add beta tester toggle to settings for channel-based update routing ([0457a3a](https://github.com/anthonyiles/SkillScout/commit/0457a3a4c9b2f1c58f5e05e439ac0aab57615d1a))
* auto-sync skill/rule files when toggling agents on a project ([#18](https://github.com/anthonyiles/SkillScout/issues/18)) ([bc39640](https://github.com/anthonyiles/SkillScout/commit/bc39640a5219bf55a16f61efb8ca51dffe0f430f))
* auto-updates via Cloudflare Worker + tauri-plugin-updater ([#22](https://github.com/anthonyiles/SkillScout/issues/22)) ([8f5246a](https://github.com/anthonyiles/SkillScout/commit/8f5246acb6c4b1f94050ee4ed7c93c96b2420402))
* implement auto-updates via Cloudflare Worker + tauri-plugin-updater ([6988caa](https://github.com/anthonyiles/SkillScout/commit/6988caaa2bbc9d947f4f3204b60fd7518d806886))
* promote local modifications to managed skills/rules ([#8](https://github.com/anthonyiles/SkillScout/issues/8)) ([d197c66](https://github.com/anthonyiles/SkillScout/commit/d197c66bb06934ad79b62356b70d01fa24e7f063))


### Bug Fixes

* address code review findings ([522bb30](https://github.com/anthonyiles/SkillScout/commit/522bb30e3692ae48ed57bd514229c71dfca80d88))
* address code review findings on beta tester toggle ([49c7f3e](https://github.com/anthonyiles/SkillScout/commit/49c7f3e1955a1c863b4f9b6e9a6badc35a467a5c))
* address CodeRabbit findings on beta tester toggle ([73cd380](https://github.com/anthonyiles/SkillScout/commit/73cd380c4e4a691b66f2152496e1c9d3cec4a2fb))
* address CodeRabbit review findings ([b193b74](https://github.com/anthonyiles/SkillScout/commit/b193b741f15562441ae4ef9ff7474bf14d8f72c2))
* address pre-PR CodeRabbit findings on feat/auto-update ([612a3a3](https://github.com/anthonyiles/SkillScout/commit/612a3a39eddea076759b2c0a1ec08f4098ebadcd))
* address second code review pass findings ([ee2df31](https://github.com/anthonyiles/SkillScout/commit/ee2df31aa899a1e589c4f1d8f2d61d5ba3a4a8c6))
* address third code review pass findings ([c48aa0b](https://github.com/anthonyiles/SkillScout/commit/c48aa0b8751836e791eda98c3c640c63844352ad))
* pass target-branch to release-please so PRs target the correct branch ([16eaaf1](https://github.com/anthonyiles/SkillScout/commit/16eaaf10aa25f8a57a14725f69e68d1f3f0af68c))
* resolve remaining legitimate CodeRabbit findings from PRs [#5](https://github.com/anthonyiles/SkillScout/issues/5)-[#8](https://github.com/anthonyiles/SkillScout/issues/8) ([#15](https://github.com/anthonyiles/SkillScout/issues/15)) ([cc12c02](https://github.com/anthonyiles/SkillScout/commit/cc12c021b6aca39a19ede760810e85a537af9690))
* resolve updater channel routing and signing pipeline ([a2e610a](https://github.com/anthonyiles/SkillScout/commit/a2e610ad59b2a9fd97e30b44c91357f0b9ef0032))
* **tests:** mock useUpdater and fix PageLayout stub slot order in SettingsView tests ([4845e46](https://github.com/anthonyiles/SkillScout/commit/4845e46bbd20a968b0fc180a254770be4d451dd0))
* update worker URL to skillscout-update.iles.sh ([aa4477b](https://github.com/anthonyiles/SkillScout/commit/aa4477b1c8818d86d29279fbf68b94c8b6e3aca2))
* use PAT for release-please to trigger build workflow ([b2ac037](https://github.com/anthonyiles/SkillScout/commit/b2ac037b61e9a03b3b07f1453118e0868bbe3fdb))
* wire unused SkillScoutError variants into their natural call sites ([ddee6e9](https://github.com/anthonyiles/SkillScout/commit/ddee6e9823b3959a08693198bfdaae9edf654444))

## [0.3.1](https://github.com/anthonyiles/SkillScout/compare/skillscout-v0.3.0...skillscout-v0.3.1) (2026-05-31)


### Bug Fixes

* resolve updater channel routing and signing pipeline ([a2e610a](https://github.com/anthonyiles/SkillScout/commit/a2e610ad59b2a9fd97e30b44c91357f0b9ef0032))

## [0.3.0](https://github.com/anthonyiles/SkillScout/compare/skillscout-v0.2.0...skillscout-v0.3.0) (2026-05-31)


### Features

* add beta tester toggle for channel-based update routing ([c098e21](https://github.com/anthonyiles/SkillScout/commit/c098e21b7b7594aee8fd261c9b14e3e1bd38bccc))
* add beta tester toggle to settings for channel-based update routing ([0457a3a](https://github.com/anthonyiles/SkillScout/commit/0457a3a4c9b2f1c58f5e05e439ac0aab57615d1a))
* auto-sync skill/rule files when toggling agents on a project ([#18](https://github.com/anthonyiles/SkillScout/issues/18)) ([bc39640](https://github.com/anthonyiles/SkillScout/commit/bc39640a5219bf55a16f61efb8ca51dffe0f430f))
* auto-updates via Cloudflare Worker + tauri-plugin-updater ([#22](https://github.com/anthonyiles/SkillScout/issues/22)) ([8f5246a](https://github.com/anthonyiles/SkillScout/commit/8f5246acb6c4b1f94050ee4ed7c93c96b2420402))
* implement auto-updates via Cloudflare Worker + tauri-plugin-updater ([6988caa](https://github.com/anthonyiles/SkillScout/commit/6988caaa2bbc9d947f4f3204b60fd7518d806886))
* promote local modifications to managed skills/rules ([#8](https://github.com/anthonyiles/SkillScout/issues/8)) ([d197c66](https://github.com/anthonyiles/SkillScout/commit/d197c66bb06934ad79b62356b70d01fa24e7f063))


### Bug Fixes

* address code review findings ([522bb30](https://github.com/anthonyiles/SkillScout/commit/522bb30e3692ae48ed57bd514229c71dfca80d88))
* address code review findings on beta tester toggle ([49c7f3e](https://github.com/anthonyiles/SkillScout/commit/49c7f3e1955a1c863b4f9b6e9a6badc35a467a5c))
* address CodeRabbit findings on beta tester toggle ([73cd380](https://github.com/anthonyiles/SkillScout/commit/73cd380c4e4a691b66f2152496e1c9d3cec4a2fb))
* address CodeRabbit review findings ([b193b74](https://github.com/anthonyiles/SkillScout/commit/b193b741f15562441ae4ef9ff7474bf14d8f72c2))
* address pre-PR CodeRabbit findings on feat/auto-update ([612a3a3](https://github.com/anthonyiles/SkillScout/commit/612a3a39eddea076759b2c0a1ec08f4098ebadcd))
* address second code review pass findings ([ee2df31](https://github.com/anthonyiles/SkillScout/commit/ee2df31aa899a1e589c4f1d8f2d61d5ba3a4a8c6))
* address third code review pass findings ([c48aa0b](https://github.com/anthonyiles/SkillScout/commit/c48aa0b8751836e791eda98c3c640c63844352ad))
* pass target-branch to release-please so PRs target the correct branch ([16eaaf1](https://github.com/anthonyiles/SkillScout/commit/16eaaf10aa25f8a57a14725f69e68d1f3f0af68c))
* resolve remaining legitimate CodeRabbit findings from PRs [#5](https://github.com/anthonyiles/SkillScout/issues/5)-[#8](https://github.com/anthonyiles/SkillScout/issues/8) ([#15](https://github.com/anthonyiles/SkillScout/issues/15)) ([cc12c02](https://github.com/anthonyiles/SkillScout/commit/cc12c021b6aca39a19ede760810e85a537af9690))
* **tests:** mock useUpdater and fix PageLayout stub slot order in SettingsView tests ([4845e46](https://github.com/anthonyiles/SkillScout/commit/4845e46bbd20a968b0fc180a254770be4d451dd0))
* update worker URL to skillscout-update.iles.sh ([aa4477b](https://github.com/anthonyiles/SkillScout/commit/aa4477b1c8818d86d29279fbf68b94c8b6e3aca2))
* use PAT for release-please to trigger build workflow ([b2ac037](https://github.com/anthonyiles/SkillScout/commit/b2ac037b61e9a03b3b07f1453118e0868bbe3fdb))
* wire unused SkillScoutError variants into their natural call sites ([ddee6e9](https://github.com/anthonyiles/SkillScout/commit/ddee6e9823b3959a08693198bfdaae9edf654444))
