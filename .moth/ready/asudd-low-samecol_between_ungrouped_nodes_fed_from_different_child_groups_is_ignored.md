A sameCol hint between two ungrouped data nodes that are fed from different groups is not honoured in some nesting shapes (found by hs412 on ghost/phase3/phase3.compact.clarc.json: `smtp sameCol mailgun` reported unsatisfied, the SMTP provider lands one column left of Mailgun).

Minimal repro (engine input): groups G, H1 (parent G), H2 (parent G); nodes v, c(G), a(H1), b(H2), dlq(G), q(G), smtp, mg; edges v->c, c->a, a->b, b->q, q->dlq, a->smtp, b->mg; hint `smtp same-col mg` is reported ignored (smtp col 4, mg col 5). With one child group (c->a, a->smtp, c->mg) it holds.

Wanted: such a hint holds whenever both nodes can share a column. Find where the two nodes get different columns (layering of the lifted level versus column expansion of the blocks) and fix it with a test that fails before.
