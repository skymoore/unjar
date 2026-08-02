# unjar 🍪

<div align="center">

[<img src="https://badges.ws/github/assets-dl/vladkens/unjar" />](https://github.com/vladkens/unjar/releases)
[<img src="https://badges.ws/github/release/vladkens/unjar" />](https://github.com/vladkens/unjar/releases)
[<img src="https://badges.ws/github/license/vladkens/unjar" />](https://github.com/vladkens/unjar/blob/main/LICENSE)
[<img src="https://badges.ws/badge/-/buy%20me%20a%20coffee/ff813f?icon=buymeacoffee&label" alt="donate" />](https://buymeacoffee.com/vladkens)

</div>

`unjar` reads cookies from local browser profiles and exports them as a
`cookies.txt` file, JSON, or a `Cookie:` header. Use it as a CLI or Rust library.

## Supported browsers

Legend: ✅ tested · 🟡 implemented, not yet tested · 🚧 not implemented.

| Browser | macOS | Linux | Windows |
| ------- | :---: | :---: | :-----: |
| Chrome | ✅ | 🟡 | 🚧 |
| Chromium / Edge / Brave | 🟡 | 🟡 | 🚧 |
| Firefox | 🟡 | 🟡 | 🟡 |
| Safari | 🚧 | — | — |

Only Chrome on macOS has been verified end-to-end so far. Linux Chromium
decryption currently relies on the `peanuts` fallback and will not decrypt
profiles that store the key in the system keyring (v11). Windows and Safari are
not implemented yet.

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

```sh
# show discovered browser profiles
unjar list

# dump x.com cookies using the default browser
unjar x.com

# select a profile by stable ID, unique display name, or path
unjar x.com --profile 'chrome:Profile 1'
unjar x.com --profile 'Work'
unjar x.com --profile /path/to/profile

# straight into a file for curl / yt-dlp
unjar x.com -o cookies.txt
curl -b cookies.txt https://x.com/...
```

`unjar list` prints each discovered profile's browser, stable ID, display name, and path. If display names are duplicated, select the profile by ID. Explicit paths may point to a profile directory or directly to its cookie database.

## Library

```sh
cargo add unjar
```

```rust
use unjar::{Browser, cookies_for_profile, find_profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let profile = find_profile(Browser::Chrome, "chrome:Default")?;
  let jar = cookies_for_profile(&profile, "x.com")?;

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

[twscrape](https://github.com/vladkens/twscrape) accepts a cookie header string
(`auth_token=...; ct0=...`) or a JSON array — both of which `unjar` emits. Log
into X in your browser, then hand the cookies straight to an account:

```sh
twscrape add_cookie my_username "$(unjar x.com -f header)"
```

> `"$(...)"` works the same in sh, bash, zsh and fish (3.4+): the whole cookie
> string is passed as a single argument.

From Python:

```python
import subprocess
from twscrape import API

cookies = subprocess.check_output(["unjar", "x.com", "-f", "header"], text=True).strip()

api = API()
await api.pool.add_account_cookies("my_username", cookies)
```

`unjar x.com` already includes the `auth_token` and `ct0` cookies that twscrape
needs, plus the rest of the session.

## Contributing

All contributions are welcome! Feel free to open an issue or submit a pull request.

## License

Distributed under the [MIT License](LICENSE).
