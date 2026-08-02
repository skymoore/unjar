# unjar 🍪

<div align="center">

[<img src="https://badges.ws/github/assets-dl/vladkens/unjar" />](https://github.com/vladkens/unjar/releases)
[<img src="https://badges.ws/github/release/vladkens/unjar" />](https://github.com/vladkens/unjar/releases)
[<img src="https://badges.ws/github/license/vladkens/unjar" />](https://github.com/vladkens/unjar/blob/main/LICENSE)
[<img src="https://badges.ws/badge/-/buy%20me%20a%20coffee/ff813f?icon=buymeacoffee&label" alt="donate" />](https://buymeacoffee.com/vladkens)

</div>

`unjar` reads cookies from local browser profiles and exports them as a `cookies.txt` file, JSON, or a `Cookie:` header. Use it as a CLI or Rust library.

## Installation

Install using [Homebrew](https://brew.sh/):

```sh
brew install vladkens/tap/unjar
```

Or install using [Cargo](https://crates.io/crates/unjar):

```sh
cargo install unjar
```

## CLI

Show discovered browser profiles:

```sh
unjar list
```

Show help without reading any cookies:

```sh
unjar
```

Dump cookies for one or more domains using Chrome by default:

```sh
unjar x.com
```

```sh
unjar x.com t.co
```

Explicitly dump every cookie from the selected browser or profile:

```sh
unjar all
```

Select a profile by ID, unique display name, or path:

```sh
unjar -p 'Profile 1' x.com
```

```sh
unjar -p 'Work' x.com
```

```sh
unjar -b chromium -p /path/to/profile x.com
```

Write cookies to a file and use it with curl or yt-dlp:

```sh
unjar -o cookies.txt x.com
curl -b cookies.txt https://x.com/...
```

`unjar list` prints each discovered profile's browser, local profile ID, default selection, and path. A unique ID or display name works without `--browser`; use `--browser` to disambiguate duplicates. An explicit path may point to a profile directory or directly to its cookie database. For a copied or otherwise unknown path, also pass `--browser` so `unjar` can select the right decryption backend.

The `header` format accepts exactly one host. It includes every cookie whose stored domain matches that host; it does not evaluate URL path, scheme, expiration, or other request attributes. Use JSON or Netscape format when exporting multiple domains or `all`.

## Library

```sh
cargo add unjar --no-default-features
```

```rust
use unjar::Browser;

fn main() -> unjar::Result<()> {
  let profile = Browser::Chrome.find_profile("Default")?;
  let jar = profile.cookies()?.domain("x.com");

  for c in jar.iter() {
    println!("{}={}", c.name, c.value);
  }

  // or hand it straight to an HTTP client
  let header = jar.to_header();
  Ok(())
}
```

## Recipes

### twscrape

[twscrape](https://github.com/vladkens/twscrape) accepts a cookie header from stdin. Log into X in your browser, then pipe the cookies straight into a local account:

```sh
unjar -f header x.com | twscrape add_cookie my_account
```

`my_account` is a local identifier in twscrape; it does not need to match the X username stored in the cookies.

When the selected profile is logged into X, `unjar x.com` includes the `auth_token` and `ct0` cookies that twscrape needs, plus the rest of the matching session cookies.

## Supported browsers

Legend: ✅ tested · 🟡 implemented, not yet tested · 🚧 not implemented.

| Browser                 | macOS | Linux | Windows |
| ----------------------- | :---: | :---: | :-----: |
| Chrome                  |  ✅   |  🟡   |   🚧    |
| Chromium / Edge / Brave |  🟡   |  🟡   |   🚧    |
| Firefox                 |  ✅   |  🟡   |   🟡    |
| Safari                  |  🚧   |   —   |    —    |

Chrome and Firefox on macOS have been verified end-to-end. Linux Chromium decryption currently relies on the `peanuts` fallback and will not decrypt profiles that store the key in the system keyring (v11). Windows and Safari are not implemented yet.

## Contributing

All contributions are welcome! Feel free to open an issue or submit a pull request.

## License

Distributed under the [MIT License](LICENSE).
