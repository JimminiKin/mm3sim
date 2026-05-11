# MM3Sim Code Organization

This document describes the modular structure of the MM3Sim codebase.

## Directory Structure

```
src/
├── main.rs          # Application entry point and app setup
├── components/      # Entity/component spawning logic
│   ├── mod.rs
│   └── vibraphone.rs # Vibraphone bar spawning functions
├── systems/         # Bevy systems and game logic
│   ├── mod.rs
│   ├── setup.rs     # Initial world setup system
│   └── marble.rs    # Marble spawning and interaction systems
├── resources/       # Game constants and configuration
│   ├── mod.rs
│   └── constants.rs # All game constants (sizes, materials, etc.)
└── utils/           # Utility functions and helpers
    └── mod.rs       # (Currently empty, for future utilities)
```

## Module Responsibilities

- **main.rs**: Contains the main function, app initialization, and system scheduling
- **components/**: Functions for spawning complex entities/components
- **systems/**: Bevy systems that run during the game loop
- **resources/**: Constants, configuration values, and shared data
- **utils/**: Helper functions, math utilities, and common operations

## Adding New Features

When adding new simulation elements:

1. **New entity types**: Add spawning functions to `components/`
2. **New game systems**: Add systems to `systems/`
3. **New constants**: Add to `resources/constants.rs`
4. **Utility functions**: Add to `utils/`
5. **Update main.rs**: Register new systems in the app setup

This structure allows for easy expansion and maintains clean separation of concerns.
