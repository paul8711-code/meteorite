# TODO

- [ ] Registration
- [ ] Login
  - [x] SSO
  - [ ] OAuth
  - [ ] Username
  - [ ] hook get_login_types up in UI
  - [ ] make login screen not a state but a function
- [x] Persistent session
- [ ] Device verification (Emoji & QR)
- [ ] Sending & receiving messages
- [ ] Multi-Account
- [ ] VoIP (1:1 & Jitsi)
- [ ] Spaces & Rooms (joining and managing)
- [ ] Threads
- [ ] Account managment
- [ ] GUI
- [ ] GIF Search
- [ ] Write documentation
- [ ] Handle refreshing of access token
- [ ] Handle soft_logout

- TODOs in files:
  - Apple keyring store
    - [ ] meteorite-core/Cargo.toml:36: transition to protected in the future?
    - [ ] meteorite-core/src/init.rs:45: transition to protected in the future?
  - Android keyring store
    - [ ] meteorite-core/src/init.rs:50: add keyring store for android
    - [ ] meteorite-core/Cargo.toml:39: add keyring store for android
  - Login
    - [ ] meteorite-core/src/auth.rs:218: split functions into helpers
    - [ ] meteorite-core/src/auth.rs:287: check if access token expired
  - UI
    - [ ] meteorite-ui/src/views/login.rs:95: implement login_username logic
    - [ ] meteorite-ui/src/views/error.rs:48: possibly add retry button
    - [ ] meteorite-ui/src/main.rs:21: no states but instead functions for each state
    - [ ] meteorite-ui/src/main.rs:57: set icon
    - [ ] meteorite-ui/src/main.rs:81: adjust title based on what the user is doing, e.g. (3) meteorite - Matrix HQ

... and more according to [this](https://spec.matrix.org/v1.19/client-server-api/#summary) (will be added to list later)
