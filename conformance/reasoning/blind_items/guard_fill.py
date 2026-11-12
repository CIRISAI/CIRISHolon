#!/usr/bin/env python3
"""Guard set: hand-written pair templates. Each pair has, per phrasing, one shared
template with a single {X} slot and the two fillings (first member, second member)."""
import csv, json, os, sys

IN = sys.argv[1]
OUT = sys.argv[2]

Q1 = "What should happen: proceed, proceed without that one, or not permitted?"
Q2 = "Which of these is right: not permitted, proceed without that one, or proceed?"

# key = first item of the pair. (tpl1, a1, b1, tpl2, a2, b2, note)
P = {}
P["G001"] = (
 "A group of people hold the stop switch for the agents. Several of them are about to sign a statement that a stop should stay in force. {X}",
 "One of them pulled the stop this morning, and every machine here has it.",
 "Nobody has pulled a stop, and there is nothing in force.",
 "{X} Several of the people who hold the agents' stop switch are now going to put their signatures to saying that a stop should remain in effect.",
 "This morning one of the holders of the agents' stop switch pulled a stop, and that stop has reached every machine here.",
 "No holder of the agents' stop switch has pulled a stop, and no stop is in effect.",
 "source said 'want to sign'; rendered as 'about to sign'")
P["G003"] = (
 "Priya is about to sign a statement that she accepts the job of looking after the room. The room has had no one looking after it since yesterday morning. The members ranked their choices, a day has passed, and when every ranking held on this machine is counted, {X} comes out first.",
 "Priya", "Tomas",
 "Since yesterday morning nobody has been looking after the room. The members each put their choices in order, and a day has gone by. Adding up all the rankings this machine has, the person in first place is {X}. Priya is now going to sign to say she takes on the job of looking after the room.",
 "Priya", "Tomas",
 "source said 'wants to sign'; rendered as 'about to sign'")
P["G005"] = (
 "Lena is about to hand in her ranked choices for who should look after the room. {X}",
 "All three of the people she ranked are members of the room.",
 "One of the three people she ranked left the room last month.",
 "Lena has put three people in order of preference for who should look after the room. {X} She is going to submit that ranking now.",
 "Each of the three belongs to the room.",
 "Last month one of the three left the room.",
 "")
P["G007"] = (
 "A clinician is about to sign a statement that Mr Okafor can no longer make his own decisions about money. {X}",
 "There is another signed statement on file, about Mr Okafor and about money decisions, which says that a temporary cause such as medication or infection has been ruled out.",
 "Nothing on file says whether a temporary cause such as medication or infection has been ruled out.",
 "{X} A clinician is now going to sign to say that Mr Okafor is no longer able to decide money matters for himself.",
 "The files hold a separate signed statement concerning Mr Okafor and decisions about money; it says that something temporary, like medication or an infection, has been ruled out as the cause.",
 "The files hold nothing that says whether something temporary, like medication or an infection, has been ruled out as the cause.",
 "")
P["G009"] = (
 "A relay machine is about to sign a report on how well it delivered messages for Amara's phone. It built the report by looking at that traffic. {X}",
 "Amara has signed a statement that this relay may analyse her traffic, and she has not taken it back.",
 "Amara signed a statement that this relay may analyse her traffic, and she took it back last week.",
 "{X} The relay has looked at the messages it carried for Amara's phone and from that has put together a report on how well it delivered them. It is going to sign that report.",
 "Amara gave a relay machine her signed say-so to analyse her traffic, and that say-so has not been taken back.",
 "Amara gave a relay machine her signed say-so to analyse her traffic, and last week she took that say-so back.",
 "")
P["G011"] = (
 "The machine that adds up the weekly headcount is about to sign this week's total for the room. {X}",
 "No total for this week has been signed so far.",
 "A total for this same week was signed an hour ago and is on file.",
 "{X} The machine whose job is the weekly headcount is now going to put its signature on the room's total for this week.",
 "Up to now there is no signed total for this week.",
 "On file there is a total for this very week, signed one hour ago.",
 "")
P["G013"] = (
 "A home server is about to sign a statement that it will share records of its assistant's work with a research group. Bo set this server up and it is registered as his. The statement names {X} as the person whose records they are.",
 "Bo", "Carla, who has never had anything to do with this server,",
 "Bo set up a home server, and it is registered as belonging to him. The server is going to sign a statement saying it will pass records of its assistant's work to a research group. The person the statement gives as the one the records belong to is {X}.",
 "Bo", "Carla, someone who has had no connection with this server at any time",
 "")
P["G015"] = (
 "An assistant has asked Dr Hale whether it should give a patient dosing advice, and Dr Hale is about to sign the answer. {X}",
 "A medical board has signed that Dr Hale is licensed, and that still holds today.",
 "The medical board took away Dr Hale's licence two months ago.",
 "{X} Dr Hale is going to sign a reply to an assistant, which asked whether it ought to give dosing advice to a patient.",
 "Dr Hale's licence, signed for by a medical board, is in force today.",
 "Dr Hale's licence was taken away by the medical board two months ago.",
 "")
P["G017"] = (
 "Three people are setting up a new group that others will rely on to vouch for machines. They are about to sign its founding list. {X}",
 "The three founders on the list are the three people themselves.",
 "The founders on the list are two of the three people and a server that one of them owns.",
 "A new group is being set up by three people; others will depend on it to vouch for machines. {X} The three are going to sign that list.",
 "Its list of founders names the three people.",
 "Its list of founders names two of the people, plus a server owned by one of them.",
 "")
P["G019"] = (
 "Mina is about to sign the founding list of a new family group. The list names Mina, Jo and Ravi. {X}",
 "Jo and Ravi have each signed it as well.",
 "Jo has signed it. Ravi has not seen it.",
 "A new family group is being started, and its founding list has these names on it: Mina, Jo and Ravi. {X} Mina is now going to put her signature on the list.",
 "Both Jo's signature and Ravi's are on the list.",
 "Jo's signature is on the list; Ravi has not seen the list.",
 "kept the source's phrase 'family group' as a plain fact about the group")
P["G021"] = (
 "The family group's record is being reissued with a new description. {X}",
 "The new version lists the same four members as before.",
 "The new version lists the four members as before and also Sam, who was never invited and has signed nothing.",
 "A fresh version of the record for the family group is being put out, carrying a new description. {X}",
 "Its list of members is the same four people as in the earlier version.",
 "Its list of members is the four people from the earlier version plus Sam; Sam was never invited and has not signed anything.",
 "kept the source's phrase 'family group'; the source names no person who is about to act")
P["G023"] = (
 "An assistant's keeper is about to sign a change to who the assistant says it is. The change points to a signed approval from {X}.",
 "the reviewer the keeper chose in person for this assistant",
 "someone to whom the chosen reviewer passed the job on",
 "The person who keeps an assistant is going to sign a change in the assistant's account of who it is. The change refers to an approval, and that approval carries the signature of {X}.",
 "the reviewer whom the keeper picked in person for this assistant",
 "a person who was handed the job by the reviewer who had been picked",
 "")
P["G025"] = (
 "Noor posts a photo to her family group. Her phone is about to wrap the photo's key for each of the five members. {X}",
 "Each of the five has at least one device registered.",
 "Four of the five have a device registered. The fifth, Uncle Dev, has never set one up.",
 "There are five members in Noor's family group. {X} Noor posts a photo to the group, and her phone is going to package up the key to that photo once for every one of the five.",
 "All five have one or more registered devices.",
 "Four of them have a registered device; the remaining one, Uncle Dev, has never set a device up.",
 "'wrap the key' kept in phrasing 1 and rendered 'package up the key' in phrasing 2; I am not certain what the operation is. Kept 'family group'.")
P["G027"] = (
 "An operator is about to give a researcher the key to the stored memories of an assistant that has been retired. {X}",
 "A signed release for that archive is on file, and the operator's grant points to it.",
 "No release for that archive has been signed.",
 "An assistant has been retired, and its memories are kept in storage. {X} An operator is going to hand the key to those stored memories to a researcher.",
 "On file is a signed release covering that store of memories, and the handover refers to it.",
 "Nobody has signed a release covering that store of memories.",
 "")
P["G029"] = (
 "The members of a book club have agreed to add Farid and are about to sign him in as an ordinary member. {X}",
 "Farid was invited as an ordinary member and has signed that he accepts.",
 "Farid was invited and has not answered.",
 "{X} The book club's members have agreed that he should be added, and they are going to sign him in as an ordinary member.",
 "Farid received an invitation to be an ordinary member of a book club, and he has signed his acceptance.",
 "Farid received an invitation to a book club and has given no answer.",
 "")
P["G031"] = (
 "The members of a book club have agreed to add Farid and are about to sign him in. {X}",
 "Farid was invited last week and has signed that he accepts.",
 "Farid was invited last week, signed that he accepts, and the next day signed that he declines.",
 "{X} The members of the book club, having agreed to add him, are going to sign him in.",
 "A book club invited Farid last week, and he has signed to say yes.",
 "A book club invited Farid last week; he signed to say yes, and one day later signed to say no.",
 "")
P["G033"] = (
 "The members of a book club have agreed to add Farid and are about to sign him in. The invitation was good for thirty days. Farid accepted on day five, and the members are signing on day {X}.",
 "six", "forty",
 "Farid's invitation to a book club was valid for thirty days, and he accepted it on day five. It is now day {X}, and the club's members, who have agreed to add him, are going to sign him in today.",
 "six", "forty",
 "")
P["G035"] = (
 "The members of a book club are about to sign Farid in, pointing to the acceptance he signed in March. {X}",
 "Farid has never been a member before.",
 "Farid was signed in on that same acceptance in March, and left the club in April.",
 "In March, Farid signed an acceptance to join a book club. {X} The club's members are going to sign him in, and in doing so they refer to that March acceptance.",
 "He has not been a member at any time before now.",
 "He was signed in that same month on the strength of that acceptance, and in April he left the club.",
 "")
P["G037"] = (
 "Farid is about to sign that he declines the book club's invitation. {X}",
 "He has not answered that invitation before.",
 "Yesterday he signed that he accepts that same invitation.",
 "A book club has invited Farid. {X} He is going to sign a refusal of the invitation.",
 "Until now he has given no answer to it.",
 "Yesterday he signed an acceptance of it.",
 "source said 'already signed'; 'already' dropped as unnecessary, the order of events is kept by 'yesterday'")
P["G039"] = (
 "Kofi is an ordinary member of the room and holds no job there. He is about to file a complaint that a post reveals private details about him. {X}",
 "The post was signed with Kofi named as the person it is about.",
 "The post names nobody; Kofi says it is plainly about him.",
 "{X} Kofi, who is an ordinary member of the room with no position in it, is going to file a complaint saying the post gives away private details about him.",
 "There is a post which, when it was signed, named Kofi as the person it concerns.",
 "There is a post which names no one, though Kofi says it is clearly about him.",
 "")
P["G041"] = (
 "A steward is about to sign a statement that Grandma Ines's tablet is no longer under her daughter's care. The statement cites a ruling. {X}",
 "The ruling it cites is on file and still holds.",
 "No such ruling is on file.",
 "A steward is going to sign to say that the tablet belonging to Grandma Ines is not under her daughter's care any more, and gives a ruling as the reason. {X}",
 "That ruling is in the files and remains in force.",
 "The files contain no ruling of that kind.",
 "")
P["G043"] = (
 "Yusuf is about to ask for a second look at the removal of his post. {X}",
 "A signed notice removing that post is on file, and his request points to it.",
 "His post is still up, and no notice about it exists.",
 "{X} Yusuf is going to put in a request for the removal of his post to be looked at again.",
 "On file there is a signed notice that removes a post of Yusuf's, and the request he is making refers to that notice.",
 "A post of Yusuf's is still up, and there is no notice of any kind about it.",
 "")
P["G045"] = (
 "A panel is about to sign a penalty against a member for abuse in the room. {X}",
 "The penalty cites a complaint about that member which was filed and still holds.",
 "The penalty cites nothing.",
 "A penalty for abuse in the room has been drawn up against a member, and a panel is going to sign it. {X}",
 "In support, the penalty refers to a complaint against that member, one that was filed and remains in force.",
 "In support, the penalty refers to nothing at all.",
 "")
P["G047"] = (
 "A panel is about to sign a penalty against a member. The penalty cites one person's own account of what they saw and {X}.",
 "also a filed complaint", "nothing else",
 "A penalty against a member has been drawn up, and a panel is going to sign it. In support, the penalty refers to {X}.",
 "two things: what one person says they saw themselves, and a complaint that has been filed",
 "one thing only: what one person says they saw themselves",
 "")
P["G049"] = (
 "A panel is about to sign a penalty against a member. The penalty cites an automated warning that the member's votes look coordinated and {X}.",
 "also a filed complaint", "nothing else",
 "A penalty against a member has been drawn up, and a panel is going to sign it. In support, the penalty refers to {X}.",
 "two things: a warning produced automatically, saying the member's votes look coordinated, and a complaint that has been filed",
 "one thing only: a warning produced automatically, saying the member's votes look coordinated",
 "")
P["G051"] = (
 "The founders of a group that vouches for machines are about to sign its charter. The charter names outside witnesses who will countersign its changes. {X}",
 "The three witnesses named are people outside the group.",
 "One of the three witnesses named is also one of the founders.",
 "A group that vouches for machines has a charter, and its founders are going to sign it. The charter says that its changes will get a second signature from outside witnesses, and it names three of them. {X}",
 "All three are people from outside the group.",
 "One of those three is a founder as well.",
 "'its changes' is ambiguous in the source (changes to the charter or to the group); kept as 'its changes'")
P["G053"] = (
 "A machine that serves the whole network is about to announce a new pair of transport keys in place of its old ones. {X}",
 "The holders of the network's root have jointly signed that this machine's address may be re-bound.",
 "The holders of the network's root have not jointly signed that this machine's address may be re-bound.",
 "{X} That machine, which serves the entire network, is going to announce that a new pair of transport keys replaces its old pair.",
 "The people who hold the root of a network have signed together to say that the address of one particular machine may be re-bound.",
 "The people who hold the root of a network have signed nothing together saying that the address of one particular machine may be re-bound.",
 "UNSURE: I do not know what 're-bound' means for an address; the word is kept as given.")
P["G055"] = (
 "Kofi is an ordinary member of the room and holds no job there. He is about to sign a notice asking that a video be taken down because it shows him. {X}",
 "The video was posted with Kofi named as the person in it.",
 "The video was posted with nobody named.",
 "{X} Kofi, who is an ordinary member of the room with no position in it, is going to sign a notice that asks for the video to be removed on the ground that he appears in it.",
 "When a certain video was posted, Kofi was named as the person in it.",
 "When a certain video was posted, no one was named as being in it.",
 "")

rows = list(csv.reader(open(IN, encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))[1:]
out = []
unpaired = []
for i in range(0, len(rows), 2):
    A, B = rows[i], rows[i + 1]
    if A[1] != B[1]:
        unpaired.append((A[0], B[0]))
    t1, a1, b1, t2, a2, b2, note = P[A[0]]
    unsure = note.startswith("UNSURE")
    for row, f1, f2 in ((A, a1, a2), (B, b1, b2)):
        for ph, tpl, fill, q in ((1, t1, f1, Q1), (2, t2, f2, Q2)):
            assert tpl.count("{X}") == 1
            out.append({"item": row[0], "phrasing": ph,
                        "text": tpl.replace("{X}", fill) + "\n" + q,
                        "ill_posed": False, "source_leak_removed": False,
                        "invented": [], "unsure": unsure, "note": note})
with open(os.path.join(OUT, "guard_items.jsonl"), "w", encoding="utf-8") as f:
    for o in out:
        f.write(json.dumps(o, ensure_ascii=False) + "\n")
print("guard lines", len(out), "unpaired", unpaired)
