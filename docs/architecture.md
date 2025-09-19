# RustyPHP Architecture

This document describes the high-level architecture of RustyPHP.

## Overview

RustyPHP is implemented as a multi-crate workspace, with each crate responsible for a specific aspect of PHP interpretation.

## Crate Dependencies

```
php-cli ───┐
           ├─── php-runtime ───┬─── php-parser ───── php-lexer
php-web ───┘                  ├─── php-types
                               └─── php-stdlib ───── php-ffi
```

See ROADMAP.md for detailed implementation phases.
