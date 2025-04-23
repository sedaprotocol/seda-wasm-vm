# Wasmer Middlewares

> [!CAUTION]
> This was taken from the 5.0.4 release of Wasmer (https://github.com/wasmerio/wasmer/tree/v5.0.4/lib/middlewares) to address a metering vulnerability without updating Wasmer.
> In the tests the Cranelift compiler was replaced with the Singlepass compiler.

The `wasmer-middlewares` crate is a collection of various useful
middlewares:

- `metering`: A middleware for tracking how many operators are
  executed in total and putting a limit on the total number of
  operators executed.

  [See the `metering`
  example](https://github.com/wasmerio/wasmer/blob/main/examples/metering.rs)
  to get a concrete and complete example.