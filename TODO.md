# TODO

- [ ] Registration
- [ ] Login
  - [x] SSO
  - [ ] OAuth
  - [ ] Username
  - [ ] hook get_login_types up in UI
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
- [ ] Code-sign application for Apple (IMPORTANT) and Windows
- [ ] Translations (weblate)
- [ ] proper-er error handling for loading screen login

- TODOs in files:
  - Login
    - [ ] meteorite-core/src/auth.rs: check if access token expired
  - UI
    - [ ] meteorite-ui/src/views/login.rs: implement login_username logic
    - [ ] meteorite-ui/src/main.rs: set icon
    - [ ] meteorite-ui/src/main.rs: adjust title based on what the user is doing, e.g. (3) meteorite - Matrix HQ

... and more according to [this](https://spec.matrix.org/v1.19/client-server-api/#summary) (will be added to list later)
