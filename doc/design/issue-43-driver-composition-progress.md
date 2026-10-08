# Issue 43: Driver composition transport boundary

The private Driver transport carries the actual retained Runtime, supervisor,
selected native vtable, Sources and WorkflowEngine with the original Task. Its
fields are private and it is neither Clone nor Deserialize. Identity checking uses
the actual Arc allocations, exact Task content and the same Engine Sources/Store/
registered Native port, without invoking public adapter callbacks or reading a
capability bit as authority.

There is no constructor or successful installation producer in this component.
`Runtime::installed_driver_composition` explicitly refuses before Source/Driver
effects while the actual marker, protected Native input/terminal and record-only
binder are absent. The type makes their intended concrete transport available to
the actual Driver implementation; it is not Native readiness or a token-based
substitute for those producers. Private fields prevent Task/Session/SQL fixtures
or a configured alias from creating a positive object. Future installation must
compose all real private ports and the same Runtime retention, not replace this
refusal with a metadata-derived constructor or an always-live callback.

Actual Driver/pre-marker advancement, known-commit OriginalMarker and PhaseLaunch,
Native register/input/ACK/settlement, complete successor validation and factual
binding remain unfinished. This checkpoint does not qualify a positive path or
the MVP. The normative contract remains the approved managed-binding and Runtime
Driver producer designs.
