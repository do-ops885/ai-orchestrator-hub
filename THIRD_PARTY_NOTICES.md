# Third Party Notices

This file contains licensing information for third-party dependencies used in the AI Orchestrator Hub.

## Backend Dependencies (Rust)

### Core Dependencies

- **tokio** (MIT): Asynchronous runtime for Rust
- **serde** (MIT OR Apache-2.0): Serialization framework
- **anyhow** (MIT OR Apache-2.0): Error handling library
- **tracing** (MIT): Logging and diagnostics library
- **uuid** (Apache-2.0 OR MIT): Universally unique identifiers
- **chrono** (MIT/Apache-2.0): Date and time library

### Web Framework

- **axum** (MIT): Web application framework
- **tower** (MIT): Modular web framework components
- **tower-http** (MIT): HTTP utilities for Tower

### Database and Storage

- **rusqlite** (MIT): SQLite database bindings
- **base64** (MIT OR Apache-2.0): Base64 encoding/decoding
- **hex** (MIT OR Apache-2.0): Hexadecimal encoding

### Cryptography and Security

- **ring** (ISC, OpenSSL, MIT): Cryptography library
- **aes-gcm** (Apache-2.0 OR MIT): AES-GCM encryption
- **argon2** (MIT OR Apache-2.0): Password hashing
- **jsonwebtoken** (MIT): JSON Web Token implementation
- **pbkdf2** (MIT OR Apache-2.0): Password-based key derivation
- **sha2** (MIT OR Apache-2.0): SHA-2 hash function

### Neural Processing (Optional)

- **ruv-fann** (MIT): Fast Artificial Neural Network library

### Compression

- **flate2** (MIT OR Apache-2.0): DEFLATE compression

### Development Tools

- **criterion** (MIT OR Apache-2.0): Statistics-driven benchmarking
- **tempfile** (MIT OR Apache-2.0): Temporary file creation

## Frontend Dependencies (TypeScript/Node.js)

### Core Framework

- **next** (MIT): React framework
- **react** (MIT): UI library
- **react-dom** (MIT): React DOM rendering

### State Management

- **zustand** (MIT): State management library

### Development Tools

- **typescript** (Apache-2.0): TypeScript compiler
- **eslint** (MIT): Linting utility
- **prettier** (MIT): Code formatter

### Testing

- **vitest** (MIT): Test framework
- **@testing-library/react** (MIT): React testing utilities
- **@testing-library/jest-dom** (MIT): Jest DOM testing utilities

### Build Tools

- **postcss** (MIT): CSS processing tool
- **tailwindcss** (MIT): Utility-first CSS framework

## License Compatibility

All third-party dependencies are licensed under permissive open-source licenses (MIT, Apache-2.0, ISC) that are compatible with the Apache License 2.0 used by this project.

### Special Considerations

- **ruv-fann**: Licensed under LGPL-2.1. This optional dependency is only included when the `advanced-neural` feature is enabled. The LGPL license requires that if you distribute modified versions of ruv-fann, you must make the source code available under the same license.

## Attribution

This project includes and depends on the following open-source projects:

- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Serde](https://serde.rs/) - Serialization framework
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Next.js](https://nextjs.org/) - React framework
- [React](https://reactjs.org/) - UI library
- [TypeScript](https://www.typescriptlang.org/) - Programming language
- [Tailwind CSS](https://tailwindcss.com/) - CSS framework

## License Notices

### Apache License 2.0

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

### MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### ISC License

```
Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

## Contact

For questions about third-party dependencies or licensing, please contact:

- Email: <legal@ai-orchestrator-hub.dev>
- GitHub Issues: [Dependency Questions](../../issues)

## Updates

This file is updated with each release to reflect changes in dependencies. Last updated: September 2025
