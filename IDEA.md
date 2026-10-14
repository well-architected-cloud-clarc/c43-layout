# A layout engine for softwaare architecture

Software architecture diagramming tools are currently creating ugly diagrams. One of the issues is that laying out nodes and connecting them is easy until it needs to be good at telling the story - showing the flows and relations.

A good storytelling diagram (in order of priority):

- Read left-to-right. For example the website's user is usually the leftmost node and a DB is the rightmost
- Gravitate important/more connected things to the middle both vertically and horizontally
- Nodes are evenly spaced. In a 5x4 layout all cells containing nodes are same size and nodes are in the center of each
- Two connections never share an exit or entry port
- Connections leave from right (preferred for data) or bottom (preferred for non-functional) (exceptionally top and never left) side (it reads left-to-right) and enter on the edge closest to the source, never on the right
- Connection ports are evenly distributed on their edge. Exception: in a diagram without groups a connection between two nodes of one row with nothing in between may move one of its ports along its side to the other port's height (keeping the order and half a slot to its neighbours), so that it runs straight instead of detouring around the row; the other ports stay on their even slots
- Connections never leave or enter on corners
- Connections do not cross, unless otherwise impossible
- Two parallel connections' segments never overlap
- Connections are only orthogonal
- Connections are as short as possible. E.g. two connected nodes should be neighbors unless that breaks previous rules
- Connections leave the port and gravitate to the middle of the liminal space immediately next to the port. Then traverse with the least turns to the middle of the liminal space next to the destination port and enter the node
- Num same type connectors < 4? Outgoung on the right, inbound on the left, unless a shorted route is achieved by bottom/top
- 4+? Spread evenly on the two sides
- Top left is reserved for title/meta and should be generally empty
- Gravitate secondary/non-functional parts to the bottom, in rare cases top right
- Seeks to be compact. For a 16-node diagram, 5x4 is obviously better than 1x20
- All nodes a square and same size, e.g. biggest required size applies to all (the biggest node's size is enough to place its outgoing ports on the right and bottom sides) 
- Allows grouping (including nested) but nodes are always globally centrally aligned both vertically and horizontally to their neighbors
- Groups and connections are drawn in the same liminal space and while connections are allowed to cross group's boundary, their parallel segments never overlap 
- Neighboring groups have the same height or width as their respective neighbours

Let's say we're designing an AWS event-driven architecture.
Nodes:
- U. A user (initiator, so left+middle)
- CF. CloudFront distribution (linked from user) so horizontally to user's right
- R53. Route53 providing TLS cert (non-functional, so below CloudFront) 
- AGW. An api gateway linked from CloudFront (data flow, so left-to-right)
- 3 Lambda functions (LA, LB, LC) linked from api gateway
- ATH. Athena accessed by Lambda C
- S3. S3 accessed by Lambda A and B and Athena (likely the rightmost)
- CW. Cloudwatch alert (non-functional so gravitates bottom/right)
- SES. SES linked by Lambda A and Cloudwatch alert (gravitates to right of Cloudwatch alert and below Lambda A)

Imagine graph paper.
Order of laying out:

1. Nodes are laid out respecting data flow and what connects to what 

something like
```
                       LA         SES


U     CF      AGW      LB         S3


      R53              LC         ATH


                                  CW
```

2. Node size is calculated. AGW has most connections. 3 < 4, so the node size is now 3x3

3. Connections are laid out
4. Connections crossings are identified, ports are rearranged, then nodes are rearranged until either no overlaps or patience runs out

5. Groups are defined, sized and drawn around the nodes


See @perfect.drawio or @perfect.drawio.png for an example

