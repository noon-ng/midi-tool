# Port naming

Port naming of class-compliant devices is confusing and platform-dependent.

macOS seems to enumerate device outs as inputs (and ins as outputs), while
Linux names both ports as ins.

## Examples

`midi-tool list` on macOS when I connect my Roland S-1:

```
Source ports:
0: IAC Driver Bus 1
1: S-1 MIDI OUT
Target ports:
0: IAC Driver Bus 1
1: S-1 MIDI IN
```

And on my Raspberry Pi:

```
Source ports:
0: Midi Through:Midi Through Port-0 14:0
1: S-1:S-1 MIDI IN 28:0
Target ports:
0: Midi Through:Midi Through Port-0 14:0
1: S-1:S-1 MIDI IN 28:0
```

## Implications

Scripts using this tool are not portable with the current exact-match search
logic.
