# FM26 live-data mapping status

Validated against FM26 build `6000.0.52.8888375` (`6000.0.52f1-fm26-05f1`) and the active Madla IL save on 2 July 2026.

## Mapped and enabled

| Field | Source | Validation |
| --- | --- | --- |
| Player unique ID | Person entity UID | Matched the FM profile IDs for the managed squad |
| Name | Person name records | Matched FM squad and profile screens |
| Date of birth and age | Person date plus active-save date | Lars Sveingard resolves to 13 Aug 1995 and age 30, matching FM |
| Nationality | Person-to-nation relationship | Lars Sveingard resolves to Norway, matching FM |
| Position familiarity | 15-byte player position map | Corrected order includes ML and MR; matched FM positions |
| Visible attributes | 54-byte player attribute map | 1–100 storage converted to FM's displayed 1–20 values and checked against FM |
| Preferred foot | Left/right-foot storage | Exposed only as Left, Right or Both |
| Player portrait | FM unique ID | Resolves standard FM26 face/icon packs by UID; clean fallback when absent |
| FM26 role catalogue | Current-build `RoleID` metadata bitmasks | In-possession role masks, duties and out-of-possession role masks extracted from FM26 metadata |
| Player playable-role fit | Position familiarity plus visible attributes | Best/playable/secondary roles are scored from mapped FM26 role metadata, not default position labels |

Own-squad records are treated as fully known because FM exposes their player information to the manager. A visible-attribute role-evidence score may be calculated for them. It is not FM current ability or potential ability.

## Deliberately blocked

- Raw current ability and potential ability are not exposed or used.
- Dirtiness, consistency, important matches, injury proneness and versatility remain hidden.
- Wider-save player attributes remain hidden even though the entity records can be indexed.
- Wider-save players are not surfaced until the `GAME_SCOUTED_PERSON` / scout-manager relationship has been validated for this build.

## Still unmapped

| Field | Reason |
| --- | --- |
| Scout ranges, report confidence, last scouted and reliability | The scouting manager exists in memory, but its player-report collection and field semantics are not yet validated |
| Second nationality | The second-nation relationship is not yet identified |
| Form and match rating | Values are not stored directly in the player object and the match-stat relationship is not yet validated |
| Contract expiry, wage and squad status | Own-squad test data is amateur/N/A and no safely validated non-amateur relationship is available |
| Valuation and transfer value range | The current-build value relationship is not validated |
| Transfer/loan status and interest | The transfer and agent-interest relationships are not validated |
| Fitness/injury status | The condition/injury relationship is not validated; hidden injury proneness is intentionally blocked |
| Live tactic slot role/duty packet offset | Role/duty bitmask catalogues are mapped, but the per-slot tactic packet is only published when it validates against 11 live tactic slots and formation-compatible roles |

Unknown fields must remain `null` and display as **Unknown**.

## Active tactic

The native `TACTICS_MANAGER` and per-save tactic creator object are detected in the active FM26 process using read-only memory access. Formation and selected XI are decoded when the active tactic manager validates. FM26 role, duty and out-of-possession role metadata masks are now catalogued for this build.

GlassScout now scans candidate tactic packets for those real FM26 bitmasks. A role/duty packet is only published when all 11 role masks resolve and the majority are compatible with the live formation slots. If the packet does not validate, GlassScout still shows the formation/selected XI and keeps role/duty labels pending rather than inventing them.
