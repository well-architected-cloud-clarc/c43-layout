A sameCol hint between two ungrouped data nodes that are fed from different groups is not honoured in some nesting shapes (found by hs412 on ghost/phase3/phase3.compact.clarc.json: `smtp sameCol mailgun` reported unsatisfied, the SMTP provider lands one column left of Mailgun).

Minimal repro (engine input): groups G, H1 (parent G), H2 (parent G); nodes v, c(G), a(H1), b(H2), dlq(G), q(G), smtp, mg; edges v->c, c->a, a->b, b->q, q->dlq, a->smtp, b->mg; hint `smtp same-col mg` is reported ignored (smtp col 4, mg col 5). With one child group (c->a, a->smtp, c->mg) it holds.

Wanted: such a hint holds whenever both nodes can share a column. Find where the two nodes get different columns (layering of the lifted level versus column expansion of the blocks) and fix it with a test that fails before.

----- AI agent updates -------

## Specification (clarified)

A `sameCol` hint between two ungrouped nodes holds whenever both can share a column, also when each of them is lined up (`sameRow`) with a member of the same group.

## Cause (the original repro was incomplete)

The task's repro alone already holds. What breaks it is the `sameRow` hints: at the level that holds the ungrouped nodes, a hint to a group member is lifted to a hint to the whole group block, so both ungrouped nodes want "the group's row" and the level's layout puts them in the same macro row, hence side by side in different columns. The real rows differ inside the block, so nothing forbids a shared column.

## Decision

After the level has been expanded and the nodes lined up with their peers inside the blocks, an ungrouped node with a `sameCol` hint to another ungrouped node joins its peer's column when the cell there is free (not taken by a node or inside a group frame) and the move lowers the number of violated hints; columns left empty are dropped.
Rejected: dropping the lifted `sameRow` hints at the level (they are what keeps the node in the group's macro row, so its row span contains the peer's row).

## Tests

- Task repro plus `smtp same-row a`, `mg same-row dlq`: smtp and mg share a column (fails before).
- The compact phase3 snapshot plus `mailgun same-col smtp`: shared column, no ignored hints (fails before).


## Verification

Build and the full test suite pass; the compact phase3 input with `mailgun same-col smtp` added renders with both in one column and no warnings besides crossings.

## Done

Ungrouped nodes with a same-col hint share a column after the level is expanded, also when each is lined up with a member of the same group. Related follow-up: up4w6 (router detours).
