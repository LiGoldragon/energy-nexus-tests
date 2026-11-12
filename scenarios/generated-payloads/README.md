# Imported payload preservation probe

Owner: Mind research Sol, 90bd85, Energy stage-one repair branch.

This is a sandbox compatibility probe, not adopted production syntax. Ethos Zero
19.2.0 at 5a5daca315ab24460ccbcd5038130c453e853e6e generated every fixture Rust
file. None was hand-patched. Explicit repeated variant payloads are the scoped
compatibility form authorized by Opus f1pk / Astra n5e0. They are removed
when the generator honors defined imported types.

`project CHECKOUT EMPTY_SANDBOX MODE` projects these fixtures into the ordinary
contract's existing, unchanged Crane Nix graph at base
7ca10ebccdc674bbd96489ceff3e2a43d32d6ec3. Run the printed Nix command on the
builder. No local compilation or provider connection is needed.

- `ordinary`: four tests, imported Failure binary and Datom roundtrips, direct
  scalar fields and a percentage-points/day value greater than 100.
- `negative`: the same tests against the generated original bare-import variant;
  compilation must fail with missing payload-constructor/pattern errors.
- `all`: fifteen tests; all thirteen lost imported-payload sites plus public and
  internal meta failures retain their values through binary and Datom roundtrips.

The all-schema harness aliases its test crate as meta_signal_energy_nexus; it is
not a separately built production meta crate or a contract-greeting proof. It
compiles all twelve generated modules, with process-owner association assertions
satisfied by named test doubles. Their method bodies panic if invoked, and no
test invokes them. It proves type paths and payload shapes in this sandbox,
not real runtime handles, calculation behavior, persistence or effectful bodies.

No test reads a credential, calls a provider, redeems a reset or changes a live
configuration. Production crate boundaries and plain-feature configurations
remain acceptance work for the later immutable implementation gate.
