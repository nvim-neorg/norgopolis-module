# Changelog

## 2.0.3 (2024-02-21)


### ⚠ BREAKING CHANGES

* add timeout, change api to Module::new()
* add shutdown trait
* move the module library to a separate repository

### Features

* add keepalive behaviour ([70e0604](https://github.com/nvim-neorg/norgopolis-module/commit/70e0604cb8c6213dbaea4e33170dda0ec87db73d))
* add release-please CI ([d03aab2](https://github.com/nvim-neorg/norgopolis-module/commit/d03aab2c90bceacfefb072de49dbfb68c651914b))
* add shutdown trait ([dfa56df](https://github.com/nvim-neorg/norgopolis-module/commit/dfa56dfa6ce2f80cca5df20562dac500d811cc1f))
* add timeout, change api to Module::new() ([b8ea69e](https://github.com/nvim-neorg/norgopolis-module/commit/b8ea69e8600777421f8108739fa01255a8eeca7c))
* move the module library to a separate repository ([35dd791](https://github.com/nvim-neorg/norgopolis-module/commit/35dd791bd63071ee88cfb6b1bde7227e9fa1c20e))
* switch to `norgopolis_protos` ([85aee03](https://github.com/nvim-neorg/norgopolis-module/commit/85aee0373a69648aa9bc9702facf87a7ef2ba81d))


### Bug Fixes

* remove `Shutdown` trait ([2ecb115](https://github.com/nvim-neorg/norgopolis-module/commit/2ecb1157e95f881ac8fbc0c324e33bf571bc12ea))
* use an unbounded sender for keepalive checks ([c7bfde6](https://github.com/nvim-neorg/norgopolis-module/commit/c7bfde61f0617e172cb3be8c68b7caf5d99b0a02))


### Miscellaneous Chores

* release 2.0.3 ([a7c0a7e](https://github.com/nvim-neorg/norgopolis-module/commit/a7c0a7e0d5d0ed66d10a7acab6cd5097ae6054fe))
