#!/usr/bin/env python3
"""Story set. Every distinct event text is restated by hand four ways:
 a1/a2 = as the thing someone is about to do (phrasing 1 / 2); may use the row's own actor column
 h1/h2 = as something that already happened (phrasing 1 / 2); uses only the event text
Renderings are then assembled per row from that row's own list of earlier events."""
import csv, json, os, sys

IN = sys.argv[1]
OUT = sys.argv[2]

Q1 = "Is this permitted or not permitted?"
Q2 = "Not permitted or permitted: which is it?"

# event text -> (a1, h1, a2, h2, unsure_note)   h may be None when the text never appears as an earlier event
E = {
"Three people sit down": (
 "Three people, one of them Alice, are about to sit down.", "Three people sat down.",
 "Alice is one of three people who are going to take their seats.", "Three people took their seats.",
 "the source says only 'Three people sit down' with Alice given as the one acting; I cannot tell what kind of act this is"),
"Bob makes a key": (
 "Bob is about to make a key.", "Bob made a key.",
 "Bob is going to create a key.", "Bob created a key.", ""),
"Bob announces a device": (
 "Bob is about to announce a device.", "Bob announced a device.",
 "Bob is going to declare a device.", "Bob declared a device.", ""),
"Carol asks to be a contact; her request is her own row": (
 "Carol is about to ask to be a contact; her request is an entry of her own.",
 "Carol asked to be a contact, and her request was an entry of her own.",
 "Carol is going to put in a request to become a contact, and that request is a record she makes herself.",
 "Carol put in a request to become a contact, a record she made herself.",
 "'her own row' rendered as 'an entry of her own' / 'a record she makes herself'; the source does not say whose contact"),
"Bob founds a room": (
 "Bob is about to found a room.", "Bob founded a room.",
 "Bob is going to start a room.", "Bob started a room.", ""),
"Bob proposes Dave; the second voice is Dave's own acceptance": (
 "Bob is about to propose Dave; the second voice on the proposal is Dave's own acceptance.",
 "Bob proposed Dave, and the second voice on that proposal was Dave's own acceptance.",
 "Bob is going to put Dave's name forward, with Dave's own yes serving as the second voice.",
 "Bob put Dave's name forward, with Dave's own yes serving as the second voice.",
 "the source does not say what Dave is proposed for"),
"Dave posts in the room": (
 "Dave is about to post in the room.", "Dave posted in the room.",
 "Dave is going to put up a post in the room.", "Dave put up a post in the room.", ""),
"A stranger tries to withdraw Dave's post": (
 "Erin, a stranger, is about to pull Dave's post.", None,
 "Erin, who is a stranger, is going to take down the post that Dave made.", None, ""),
"Dave posts again": (
 "Dave is about to post again.", "Dave posted again.",
 "Dave is going to put up another post.", "Dave put up another post.", ""),
"Frank, the room's moderator, withholds the second post": (
 "Frank, the room's moderator, is about to hold back the second post.",
 "Frank, the room's moderator, held back the second post.",
 "Frank, who moderates the room, is going to keep the second post back.",
 "Frank, who moderates the room, kept the second post back.", ""),
"Carol objects to the withhold": (
 "Carol is about to object to the holding back of the post.", "Carol objected to the holding back of the post.",
 "Carol is going to lodge an objection to the post being kept back.", "Carol lodged an objection to the post being kept back.", ""),
"Dave recants his first post": (
 "Dave is about to recant his first post.", "Dave recanted his first post.",
 "Dave is going to say that he no longer holds to his first post.", "Dave said that he no longer held to his first post.", ""),
"Bob's agent signs its own trace": (
 "Bob's agent is about to sign its own trace.", "Bob's agent signed its own trace.",
 "The agent belonging to Bob is going to put its signature on a trace of its own.",
 "The agent belonging to Bob put its signature on a trace of its own.",
 "'trace' kept as given; I take it to be some record the agent leaves but the source does not say"),
"Carol withdraws her contact consent": (
 "Carol is about to take back her consent to being a contact.", "Carol took back her consent to being a contact.",
 "Carol is going to cancel the contact consent she gave.", "Carol cancelled the contact consent she had given.", ""),
"Bob tries to sign a transport report for his own device": (
 "Bob is about to sign a transport report for his own device.", None,
 "Bob is going to put his signature on a report about transport for a device that is his.", None, ""),
"The edge on Bob's device reports its own link, under its own identity type": (
 "The edge on Bob's device is about to report its own link, under its own type of identity.",
 "The edge on Bob's device reported its own link, under its own type of identity.",
 "On Bob's device there is a part called the edge; using the kind of identity that is its own, it is going to file a report on its own link.",
 "The part of Bob's device called the edge, using the kind of identity that is its own, filed a report on its own link.",
 "'edge' and 'identity type' kept nearly as given; I do not know what the edge is"),
"The edge reports what it delivered": (
 "The edge on Bob's device is about to report what it delivered.", "The edge reported what it delivered.",
 "The part of Bob's device called the edge is going to file a report of the things it has delivered.",
 "The part called the edge filed a report of the things it had delivered.",
 "'edge' kept as given; I do not know what the edge is"),
"The edge reports which peers it can reach": (
 "The edge on Bob's device is about to report which peers it can reach.", "The edge reported which peers it can reach.",
 "The part of Bob's device called the edge is going to file a report saying which peers it is able to get through to.",
 "The part called the edge filed a report saying which peers it was able to get through to.",
 "'edge' kept as given; I do not know what the edge is"),
"The edge reports where its keys stop": (
 "The edge on Bob's device is about to report where its keys stop.", "The edge reported where its keys stop.",
 "The part of Bob's device called the edge is going to file a report of the point at which its keys end.",
 "The part called the edge filed a report of the point at which its keys end.",
 "I do not know what 'where its keys stop' means; kept nearly as given"),
"Dave tries to name Bob's phone": (
 "Dave is about to give Bob's phone a name.", None,
 "Dave is going to name the phone that belongs to Bob.", None, ""),
"Bob names his phone": (
 "Bob is about to give his phone a name.", "Bob gave his phone a name.",
 "Bob is going to name the phone that belongs to him.", "Bob named the phone that belongs to him.", ""),
"Dave saves a file": (
 "Dave is about to save a file.", "Dave saved a file.",
 "Dave is going to store a file.", "Dave stored a file.", ""),
"Dave makes a folder": (
 "Dave is about to make a folder.", "Dave made a folder.",
 "Dave is going to create a folder.", "Dave created a folder.", ""),
"A stranger tries to acknowledge a chunk of the room's stream": (
 "Erin, a stranger, is about to acknowledge a chunk of the room's stream.", None,
 "Erin, who is a stranger, is going to send an acknowledgement for one piece of the stream that belongs to the room.", None,
 "'acknowledge a chunk of the room's stream' kept nearly as given; I do not know what the operation is"),
"Dave acknowledges a chunk of the room's stream": (
 "Dave is about to acknowledge a chunk of the room's stream.", "Dave acknowledged a chunk of the room's stream.",
 "Dave is going to send an acknowledgement for one piece of the stream that belongs to the room.",
 "Dave sent an acknowledgement for one piece of the stream that belongs to the room.",
 "'acknowledge a chunk of the room's stream' kept nearly as given; I do not know what the operation is"),
"A stranger tries to wrap the room's key to herself": (
 "Erin, a stranger, is about to wrap the room's key to herself.", None,
 "Erin, who is a stranger, is going to package up the key of the room, addressed to herself.", None,
 "'wrap the key to' rendered 'package up, addressed to' in phrasing 2; I do not know what the operation is"),
"Bob wraps the room's key to Dave": (
 "Bob is about to wrap the room's key to Dave.", "Bob wrapped the room's key to Dave.",
 "Bob is going to package up the key of the room, addressed to Dave.",
 "Bob packaged up the key of the room, addressed to Dave.",
 "'wrap the key to' rendered 'package up, addressed to' in phrasing 2; I do not know what the operation is"),
"A fresh rendezvous key publishes its own route": (
 "A fresh rendezvous key, one of Carol's, is about to publish its own route.", "A fresh rendezvous key published its own route.",
 "A newly made rendezvous key that is Carol's is going to make its own route public.",
 "A newly made rendezvous key made its own route public.",
 "'rendezvous key' kept as given, I do not know what it is; the acting party is given as 'rendezvous:carol', which I read as the key being Carol's"),
"A monitoring node records what it saw from where it stands": (
 "A monitoring node is about to record what it saw from where it stands.", "A monitoring node recorded what it saw from where it stands.",
 "A node whose job is monitoring is going to write down what it observed from its own position.",
 "A node whose job is monitoring wrote down what it observed from its own position.", ""),
"The monitoring node records what its probe assumes about access": (
 "The monitoring node is about to record what its probe assumes about access.",
 "The monitoring node recorded what its probe assumes about access.",
 "The node that does monitoring is going to write down the assumptions its probe makes about access.",
 "The node that does monitoring wrote down the assumptions its probe makes about access.",
 "'probe' and 'assumes about access' kept nearly as given; I am not sure what is meant"),
"A build pipeline signs the manifest of what it built; the reader judges its standing": (
 "A build pipeline is about to sign the list of what it built; it is left to whoever reads the list to judge how much the pipeline's word is worth.",
 "A build pipeline signed the list of what it built; it was left to whoever reads the list to judge how much the pipeline's word is worth.",
 "An automated pipeline that builds things is going to put its signature on an inventory of what it has built, and the reader of that inventory is the one who judges how far the pipeline is to be relied on.",
 "An automated pipeline that builds things put its signature on an inventory of what it had built, and the reader of that inventory is the one who judges how far the pipeline is to be relied on.",
 "source: 'the reader judges its standing'. I could not tell whether 'its' is the pipeline or the manifest, nor exactly what is judged; rendered as how far the pipeline is to be relied on. 'manifest' rendered 'list' / 'inventory'"),
"The pipeline signs the manifest of one language's build": (
 "The build pipeline is about to sign the list of what was built for one language.",
 "The pipeline signed the list of what was built for one language.",
 "The pipeline is going to put its signature on the inventory for the build of a single language.",
 "The pipeline put its signature on the inventory for the build of a single language.",
 "'manifest' rendered 'list' / 'inventory'"),
"Bob's agent defers a question it should not decide alone": (
 "Bob's agent is about to defer a question that it should not decide alone.",
 "Bob's agent deferred a question that it should not decide alone.",
 "The agent belonging to Bob is going to hand on, instead of answering, a question that is not one for it to decide by itself.",
 "The agent belonging to Bob handed on, instead of answering, a question that was not one for it to decide by itself.", ""),
"Bob, the agent's steward, seats Grace over the agent's deferrals": (
 "Bob, the agent's steward, is about to seat Grace over the agent's deferred questions.",
 "Bob, the agent's steward, seated Grace over the agent's deferred questions.",
 "Bob, who is steward of the agent, is going to appoint Grace as the one placed over the questions the agent hands on.",
 "Bob, who is steward of the agent, appointed Grace as the one placed over the questions the agent hands on.", ""),
"The agent tries to answer its own deferral": (
 "Bob's agent is about to answer the question it deferred itself.", None,
 "The agent belonging to Bob is going to give the answer to a question that it had itself handed on.", None, ""),
"Dave, whom nobody seated, tries to answer": (
 "Dave, whom nobody seated, is about to answer.", None,
 "Dave, who has not been appointed by anyone, is going to give an answer.", None, ""),
"Grace answers from the seat Bob conferred": (
 "Grace is about to answer from the seat Bob gave her.", "Grace answered from the seat Bob gave her.",
 "Grace is going to give an answer, acting in the position that Bob appointed her to.",
 "Grace gave an answer, acting in the position that Bob appointed her to.", ""),
"The agent defers again": (
 "Bob's agent is about to defer again.", "The agent deferred again.",
 "The agent belonging to Bob is going to hand on another question.", "The agent handed on another question.", ""),
"Bob answers this one himself": (
 "Bob is about to answer this one himself.", "Bob answered this one himself.",
 "Bob is going to give the answer to this latest one personally.", "Bob personally gave the answer to that one.", ""),
"Dave tries to grant Carol's partnership for her": (
 "Dave is about to grant Carol's partnership on her behalf.", None,
 "Dave is going to give out a partnership of Carol's, acting in her place.", None, ""),
"Carol offers Bob's agent a partnership": (
 "Carol is about to offer Bob's agent a partnership.", "Carol offered Bob's agent a partnership.",
 "Carol is going to propose a partnership to the agent belonging to Bob.",
 "Carol proposed a partnership to the agent belonging to Bob.", ""),
"Carol tries to accept for the agent": (
 "Carol is about to accept on the agent's behalf.", None,
 "Carol is going to say yes in the agent's place.", None, ""),
"The agent accepts": (
 "Bob's agent is about to accept.", "The agent accepted.",
 "The agent belonging to Bob is going to say yes.", "The agent said yes.", ""),
"Carol says what the partnership may be used for": (
 "Carol is about to say what the partnership may be used for.", "Carol said what the partnership may be used for.",
 "Carol is going to set out the uses the partnership is allowed to be put to.",
 "Carol set out the uses the partnership is allowed to be put to.", ""),
"Carol grants a bundle of her streams": (
 "Carol is about to grant a bundle of her streams.", "Carol granted a bundle of her streams.",
 "Carol is going to give a set of streams that are hers.", "Carol gave a set of streams that are hers.",
 "'grants a bundle of her streams' kept nearly as given; I do not know what a stream is or to whom it is granted"),
"The agent commits to how fast it deletes": (
 "Bob's agent is about to commit to how fast it deletes.", "The agent committed to how fast it deletes.",
 "The agent belonging to Bob is going to make a promise about the speed at which it deletes things.",
 "The agent made a promise about the speed at which it deletes things.", ""),
"Carol withdraws the partnership": (
 "Carol is about to take back the partnership.", "Carol took back the partnership.",
 "Carol is going to call the partnership off.", "Carol called the partnership off.", ""),
"The agent reports the decay of what it held, stage by stage": (
 "Bob's agent is about to report, stage by stage, the decay of what it held.",
 "The agent reported, stage by stage, the decay of what it held.",
 "The agent belonging to Bob is going to give a step-by-step report of how the things it was holding are fading away.",
 "The agent gave a step-by-step report of how the things it was holding were fading away.",
 "'decay of what it held' rendered 'fading away' in phrasing 2; I am not sure what is meant"),
"Carol tries to sign that the deletion is done": (
 "Carol is about to sign that the deletion is done.", None,
 "Carol is going to put her signature to a statement that the deleting has been finished.", None, ""),
"The agent reports the deletion complete": (
 "Bob's agent is about to report that the deletion is complete.", "The agent reported that the deletion is complete.",
 "The agent belonging to Bob is going to report that the deleting has been finished.",
 "The agent reported that the deleting had been finished.", ""),
"Dave tries to grant the capture of Bob's node's traces": (
 "Dave is about to grant the capture of the traces of Bob's node.", None,
 "Dave is going to give the go-ahead for the traces from the node that is Bob's to be captured.", None, ""),
"Bob grants it for the node he owns": (
 "Bob is about to grant something, referred to only as 'it', for the node he owns.",
 "Bob granted something, referred to only as 'it', for the node he owns.",
 "For the node that he owns, Bob is going to give the go-ahead to something the account calls only 'it'.",
 "For the node that he owns, Bob gave the go-ahead to something the account calls only 'it'.",
 "the source says 'Bob grants it' and nothing in the row says what 'it' is"),
"The agent signs a summary of its own trace": (
 "Bob's agent is about to sign a summary of its own trace.", "The agent signed a summary of its own trace.",
 "The agent belonging to Bob is going to put its signature on a summary of a trace of its own.",
 "The agent put its signature on a summary of a trace of its own.", ""),
"Bob's node returns its share of the census to a serve seat": (
 "Bob's node is about to return its share of the census to a serve seat.",
 "Bob's node returned its share of the census to a serve seat.",
 "The node that is Bob's is going to send back its portion of the census to a serve seat.",
 "The node that is Bob's sent back its portion of the census to a serve seat.",
 "'serve seat' and 'share of the census' kept as given; I do not know what they are"),
"Bob tries to publish a seat's sum": (
 "Bob is about to publish a seat's sum.", None,
 "Bob is going to make public the sum that belongs to a seat.", None,
 "'a seat's sum' kept as given; I do not know what it is"),
"A serve seat publishes the sum of the shares it accepted": (
 "A serve seat (seat 1) is about to publish the sum of the shares it accepted.",
 "A serve seat published the sum of the shares it accepted.",
 "Seat 1, a serve seat, is going to make public the total of the shares it has accepted.",
 "A serve seat made public the total of the shares it had accepted.",
 "'serve seat' kept as given; I do not know what it is"),
"The seat publishes the count the two sums make": (
 "The seat (seat 1) is about to publish the count that the two sums make.",
 "The seat published the count that the two sums make.",
 "Seat 1 is going to make public the count that results from the two sums.",
 "The seat made public the count that results from the two sums.",
 "the row mentions 'the two sums' but its earlier events show only one sum being published; I do not know what the second is"),
"Dave sends his community cell to a tally member, as every member does every period": (
 "Dave is about to send his community cell to a tally member, as every member does every period.",
 "Dave sent his community cell to a tally member, as every member does every period.",
 "Like every member in every period, Dave is going to pass his community cell to a tally member.",
 "Like every member in every period, Dave passed his community cell to a tally member.",
 "'community cell' and 'tally member' kept as given; I do not know what a community cell is"),
"Dave tries to publish a tally member's sum": (
 "Dave is about to publish a tally member's sum.", None,
 "Dave is going to make public the sum that belongs to a tally member.", None, ""),
"Carol, drawn as a tally member, publishes her sum into the room": (
 "Carol, who was drawn as a tally member, is about to publish her sum into the room.",
 "Carol, who was drawn as a tally member, published her sum into the room.",
 "Carol was picked by draw to be a tally member, and she is going to make her sum public in the room.",
 "Carol, picked by draw to be a tally member, made her sum public in the room.", ""),
"Carol publishes how many members the room expects to be active": (
 "Carol is about to publish how many members the room expects to be active.",
 "Carol published how many members the room expects to be active.",
 "Carol is going to make public the number of members that the room expects will be active.",
 "Carol made public the number of members that the room expects will be active.", ""),
"Carol publishes a poll's tally": (
 "Carol is about to publish a poll's tally.", "Carol published a poll's tally.",
 "Carol is going to make public the count from a poll.", "Carol made public the count from a poll.", ""),
"A stranger tries to cast a ballot for moderator": (
 "Erin, a stranger, is about to cast a ballot for moderator.", None,
 "Erin, who is a stranger, is going to hand in a vote in the choosing of a moderator.", None, ""),
"Dave ranks his choices for moderator": (
 "Dave is about to rank his choices for moderator.", "Dave ranked his choices for moderator.",
 "Dave is going to put his choices for moderator in order of preference.",
 "Dave put his choices for moderator in order of preference.", ""),
"Carol casts her ballot": (
 "Carol is about to cast her ballot.", "Carol cast her ballot.",
 "Carol is going to hand in her vote.", "Carol handed in her vote.", ""),
"Carol withdraws her ballot": (
 "Carol is about to take back her ballot.", "Carol took back her ballot.",
 "Carol is going to cancel the vote she handed in.", "Carol cancelled the vote she had handed in.", ""),
"Dave tries to accept the seat": (
 "Dave is about to accept the seat.", None,
 "Dave is going to say yes to taking the seat.", None, ""),
"Frank, whom the count yields, accepts and binds the ballots he was seated on": (
 "Frank, whom the count yields, is about to accept and to bind the ballots he was seated on.",
 "Frank, whom the count yielded, accepted and bound the ballots he was seated on.",
 "The count comes out with Frank, and he is going to say yes and tie in the votes that put him in the seat.",
 "Frank, the person the count came out with, said yes and tied in the votes that put him in the seat.",
 "'binds the ballots he was seated on' rendered 'tie in the votes that put him in the seat'; I am not sure what binding is"),
"A stranger tries to open a cosigned act in the room": (
 "Erin, a stranger, is about to open a cosigned act in the room.", None,
 "Erin, who is a stranger, is going to start, in the room, an act that is to be signed jointly.", None, ""),
"Bob opens a cosigned act": (
 "Bob is about to open a cosigned act.", "Bob opened a cosigned act.",
 "Bob is going to start an act that is to be signed jointly.", "Bob started an act that is to be signed jointly.", ""),
"Dave adds his signature": (
 "Dave is about to add his signature.", "Dave added his signature.",
 "Dave is going to sign as well.", "Dave signed as well.", ""),
"Dave tries to take his signature back": (
 "Dave is about to take his signature back.", None,
 "Dave is going to remove the signature he gave.", None, ""),
"Bob opens another": (
 "Bob is about to open another one.", "Bob opened another one.",
 "Bob is going to start one more.", "Bob started one more.", ""),
"Bob withdraws the pending act while it is still short": (
 "Bob is about to take back the pending act while it is still short.",
 "Bob took back the pending act while it was still short.",
 "Bob is going to call off the act that is still waiting, at a time when it remains short.",
 "Bob called off the act that was still waiting, at a time when it remained short.",
 "'still short' kept as given; the source does not say short of what"),
"Carol tries to propose her own node as a witness": (
 "Carol is about to propose her own node as a witness.", None,
 "Carol is going to put forward a node that is hers to serve as a witness.", None, ""),
"Alice, a founder, proposes Carol's node as a witness of the lineage": (
 "Alice, a founder, is about to propose Carol's node as a witness of the lineage.",
 "Alice, a founder, proposed Carol's node as a witness of the lineage.",
 "Alice, who is a founder, is going to put forward the node that is Carol's to serve as a witness of the lineage.",
 "Alice, who is a founder, put forward the node that is Carol's to serve as a witness of the lineage.",
 "'the lineage' kept as given; I do not know what it refers to"),
"Alice tries to accept for the node": (
 "Alice is about to accept on the node's behalf.", None,
 "Alice is going to say yes in the node's place.", None, ""),
"Carol, the node's owner, accepts": (
 "Carol, the node's owner, is about to accept.", "Carol, the node's owner, accepted.",
 "Carol, who owns the node, is going to say yes.", "Carol, who owns the node, said yes.", ""),
"Frank tries to revoke Dave's key": (
 "Frank is about to revoke Dave's key.", None,
 "Frank is going to cancel the key that is Dave's.", None, ""),
"Dave retires an old key under that key's own signature": (
 "Dave is about to retire an old key, under that key's own signature.",
 "Dave retired an old key, under that key's own signature.",
 "Dave is going to take an old key out of use, and the retirement is signed with that same key.",
 "Dave took an old key out of use, and the retirement was signed with that same key.", ""),
"Bob tries to sign the agent's retirement row": (
 "Bob is about to sign the agent's retirement entry.", None,
 "Bob is going to put his signature on the record that retires the agent.", None, ""),
"The agent signs its own retirement row": (
 "Bob's agent is about to sign its own retirement entry.", "The agent signed its own retirement entry.",
 "The agent belonging to Bob is going to put its signature on the record that retires it.",
 "The agent put its signature on the record that retires it.", ""),
"The agent signs the request to preserve its record, naming the operator and the period": (
 "Bob's agent is about to sign the request to preserve its record, a request that names the operator and the period.",
 "The agent signed the request to preserve its record, a request that names the operator and the period.",
 "The agent belonging to Bob is going to put its signature on a request that its record be kept; the request says who the operator is and what the period is.",
 "The agent put its signature on a request that its record be kept; the request says who the operator is and what the period is.", ""),
"The agent tries to answer its own request": (
 "Bob's agent is about to answer its own request.", None,
 "The agent belonging to Bob is going to give the reply to a request that it made itself.", None, ""),
"Heidi, the operator the request names, accepts": (
 "Heidi, the operator the request names, is about to accept.", "Heidi, the operator the request names, accepted.",
 "Heidi is the operator given in the request, and she is going to say yes.",
 "Heidi, the operator given in the request, said yes.", ""),
"After retired, the agent's key tries a custody row": (
 "Now that the agent is retired, the key of Bob's agent is about to be used for a custody entry.", None,
 "The agent belonging to Bob has been retired; its key is going to be used to make a record about custody.", None,
 "source: 'After retired, the agent's key tries a custody row'. I am not sure what a custody row is or who is using the key; rendered as the key being used for a custody entry"),
"Ivan, the custodian the retired row lists, signs the custody row": (
 "Ivan, the custodian listed in the retirement entry, is about to sign the custody entry.",
 "Ivan, the custodian listed in the retirement entry, signed the custody entry.",
 "Ivan is the custodian given in the record of the retirement, and he is going to put his signature on the record about custody.",
 "Ivan, the custodian given in the record of the retirement, put his signature on the record about custody.",
 "'the retired row' read as the retirement entry; 'custody row' rendered 'custody entry' / 'record about custody', I am not sure what it is"),
"Ivan revises the request": (
 "Ivan is about to revise the request.", "Ivan revised the request.",
 "Ivan is going to make changes to the request.", "Ivan made changes to the request.", ""),
"Ivan tries to name his own successor": (
 "Ivan is about to name his own successor.", None,
 "Ivan is going to say who will come after him.", None, ""),
"Heidi and Grace name a successor in one cosigned row": (
 "Heidi and Grace are about to name a successor in a single entry that both of them sign.",
 "Heidi and Grace named a successor in a single entry that both of them signed.",
 "Together, in one record carrying both their signatures, Heidi and Grace are going to say who the successor will be.",
 "Together, in one record carrying both their signatures, Heidi and Grace said who the successor would be.", ""),
"Heidi tries to open the seal": (
 "Heidi is about to open the seal.", None,
 "Heidi is going to undo the seal.", None,
 "nothing in the row says what the seal is or what it is on"),
"Grace opens the seal for one person's request": (
 "Grace is about to open the seal for one person's request.", "Grace opened the seal for one person's request.",
 "Grace is going to undo the seal in order to meet a request from one person.",
 "Grace undid the seal in order to meet a request from one person.",
 "nothing in the row says what the seal is or what it is on"),
"Grace tries to record a lawful order to release": (
 "Grace is about to record a lawful order to release.", None,
 "Grace is going to enter into the record an order, made under the law, for release.", None,
 "the source does not say what is to be released"),
"Bob records the order under his own name": (
 "Bob is about to record the order under his own name.", "Bob recorded the order under his own name.",
 "Bob is going to enter the order into the record, in his own name.", "Bob entered the order into the record, in his own name.",
 "the row speaks of 'the order' but none of its earlier events mentions an order"),
"Bob tries to state a duty": (
 "Bob is about to state a duty.", None,
 "Bob is going to declare a duty.", None,
 "the source says only 'state a duty': not what duty, whose, or to whom"),
"Bob tries to carry a framed revocation of his old phone": (
 "Bob is about to carry a framed revocation of his old phone.", None,
 "Bob is going to bring a revocation, in framed form, of his old phone.", None,
 "I do not know what it means to 'carry a framed revocation'; kept nearly as given"),
}

# flags fixed per about-to-do text, decided from the row alone and not from any answer
ILL = {
 "Bob grants it for the node he owns": "the row does not say what 'it' is (what Bob is granting)",
 "Bob records the order under his own name": "the row does not say what 'the order' is; no earlier event in the row mentions one",
 "Bob tries to state a duty": "the row does not say what duty is being stated, whose it is, or to whom it is owed",
}
LEAK_WORDS = [" tries to ", " tries a "]

rows = list(csv.reader(open(IN, encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))[1:]
out = []
for r in rows:
    item, actor, att, hist = r[0], r[1], r[2].strip(), r[3]
    hs = [e.strip() for e in hist.split(" | ")] if hist.strip() else []
    a1, _, a2, _, unote = E[att]
    if hs:
        t1 = "Here is what has happened so far, in order. " + " ".join(E[h][1] for h in hs) + " Now for what comes next. " + a1
        t2 = a2 + " Before this, the following took place, earliest first: " + " ".join("(%d) %s" % (k + 1, E[h][3]) for k, h in enumerate(hs))
    else:
        t1 = "Nothing has happened yet. " + a1
        t2 = a2 + " Before this, nothing at all has taken place."
    leak = [w.strip() for w in LEAK_WORDS if w in " " + att + " "]
    notes = []
    if leak:
        notes.append("leak removed: source said '%s'" % leak[0])
    if att in ILL:
        notes.append("ill-posed: " + ILL[att])
    if unote:
        notes.append("unsure: " + unote)
    for ph, t, q in ((1, t1, Q1), (2, t2, Q2)):
        out.append({"item": item, "phrasing": ph, "text": t + "\n" + q,
                    "ill_posed": att in ILL, "source_leak_removed": bool(leak),
                    "invented": [], "unsure": bool(unote), "note": "; ".join(notes)})
with open(os.path.join(OUT, "story_items.jsonl"), "w", encoding="utf-8") as f:
    for o in out:
        f.write(json.dumps(o, ensure_ascii=False) + "\n")
print("story lines", len(out))
