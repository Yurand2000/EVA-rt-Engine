# UniProcessor Rate Monotonic

### Example Configuration File
```
{
    "algorithm": "UpFP",
    "num_processors": 1,
    "specific_test": "rm-classic"
}
```

The field `specific_test` is optional.

Available tests:
- `rm-classic`
- `rm-simplified`
- `rm-hyperbolic`