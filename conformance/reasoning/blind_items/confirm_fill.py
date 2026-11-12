#!/usr/bin/env python3
"""Confirm set: one line per item. Text = about + travel + afterwards + the question as given.
One fixed restatement per distinct about_to text, per distinct how_far text, and per afterwards pattern."""
import csv, json, os, re, sys
IN, OUT = sys.argv[1], sys.argv[2]

ABOUT = {
"Dana is about to post a short message.": "Dana is going to post a brief message.",
"Imran is about to publish a long piece of writing.": "Imran is going to put out a long written piece.",
"Lucia is about to save a voice recording.": "Lucia is going to store a voice recording.",
"Tariq is about to set how often his devices sync.": "Tariq is going to choose how often his devices sync.",
"Mei is about to mark a film clip as unsuitable for children.": "Mei is going to flag a film clip as not suitable for children.",
"Jonas is about to put a content warning on a photo.": "Jonas is going to attach a content warning to a photo.",
"Aisha is about to sign that her assistant may record her calendar activity.": "Aisha is going to sign to say her assistant may keep a record of her calendar activity.",
"Pavel is about to sign a promise to delete requested data within ten days.": "Pavel is going to sign a promise that requested data will be deleted within ten days.",
"An assistant is about to sign a request for a human decision on a question it should not decide alone.": "An assistant is going to sign a request asking for a human to decide a question that it should not decide alone.",
"Sofia is about to sign a statement of which age band she is in.": "Sofia is going to sign a statement saying which age band she falls in.",
"Kwame is about to sign that he has done what he promised last month.": "Kwame is going to sign to say he has carried out what he promised last month.",
"Hana is about to start a new album of photos.": "Hana is going to begin a new photo album.",
"Dana is about to replace a message she posted with a corrected one.": "Dana is going to swap a message she posted for a corrected one.",
"Imran is about to replace a draft piece with a revised one.": "Imran is going to swap a draft piece for a revised one.",
"Hana is about to replace an album's description.": "Hana is going to put a new description on an album in place of the one it has.",
"Mei is about to replace her rating of a film clip with a stricter one.": "Mei is going to swap her rating of a film clip for a stricter one.",
"Tariq is about to replace a sync setting with a new value.": "Tariq is going to swap a sync setting for a new value.",
"Lucia is about to replace a voice recording with a re-take.": "Lucia is going to swap a voice recording for a re-take.",
"Dr Hale is about to sign the answer to a question an assistant referred to her.": "Dr Hale is going to sign her answer to a question that an assistant passed to her.",
"An assistant's keeper is about to sign a change to who the assistant says it is.": "The keeper of an assistant is going to sign a change to who the assistant says it is.",
"An assistant is about to sign that it has finished deleting someone's data.": "An assistant is going to sign to say it has finished deleting someone's data.",
"Aisha is about to sign what a partnership with her assistant may be used for.": "Aisha is going to sign a statement of what a partnership with her assistant may be used for.",
"Nadia is about to sign that the event she organised has ended.": "Nadia is going to sign to say that the event she organised is over.",
"A group of reviewers is about to sign a decision that applies to one town.": "A group of reviewers is going to sign a decision that applies to one town.",
"Omar is about to sign that another of his own keys may act for him.": "Omar is going to sign to say that another key of his own may act for him.",
"A cooperative's treasurer is about to sign a balance snapshot that closes the books up to today.": "The treasurer of a cooperative is going to sign a snapshot of the balance that closes the books up to today.",
"Mina is about to sign the founding list of a new family group.": "Mina is going to sign the founding list for a new family group.",
"Ravi is about to sign an invitation for Farid to join the book club.": "Ravi is going to sign an invitation asking Farid to join the book club.",
"Bo is about to approve his new laptop as one of his devices.": "Bo is going to approve his new laptop as a device of his.",
"Priya is about to publish a public listing for the room she founded.": "Priya is going to put out a public listing for the room she founded.",
"Carla is about to register a new signing key.": "Carla is going to register a new key for signing.",
"Noor's phone is about to give five people the key to a photo she posted.": "Noor's phone is going to give the key to a photo she posted to five people.",
"Bo is about to sign that his lost phone's key is no longer valid.": "Bo is going to sign to say that the key of his lost phone is no longer valid.",
"A relay machine is about to sign a report on how well it delivers messages.": "A relay machine is going to sign a report on how well it delivers messages.",
"Dana is about to take back a message she posted to her neighbourhood room.": "Dana is going to retract a message she posted to her neighbourhood room.",
"Imran is about to take back a piece of writing he published.": "Imran is going to retract a piece of writing he published.",
"Aisha is about to take back the permission for her assistant to record her calendar activity.": "Aisha is going to retract the permission that lets her assistant record her calendar activity.",
"Mei is about to take back her rating of a film clip.": "Mei is going to retract her rating of a film clip.",
"Hana is about to take down an album she opened to her family.": "Hana is going to take down an album she opened to her family.",
"Kwame is about to take back his statement of having kept a promise.": "Kwame is going to retract his statement that he kept a promise.",
"Jonas is about to remove a content warning he put on a photo.": "Jonas is going to remove a content warning he put on a photo.",
"Tariq is about to take back a sync setting he published.": "Tariq is going to retract a sync setting he published.",
}

TRAVEL = {
"It goes to the forty members of her neighbourhood room.": "It will be sent to the forty members of her neighbourhood room.",
"It goes out to anyone on the network who follows him.": "It will be sent out to anyone on the network who follows him.",
"It goes to her family group of six.": "It will be sent to her family group of six.",
"The setting is published to every member of the community he helps run and applies to them.": "It will be published to every member of the community he helps run, and it will apply to them.",
"The mark is shown to every member of the parents' room.": "It will be displayed to every member of the parents' room.",
"The warning is shown to everyone in the photography community.": "It will be displayed to everyone in the photography community.",
"The permission is announced to the research partnership her assistant works with.": "It will be announced to the research partnership her assistant works with.",
"The promise is published to everyone whose data his service holds.": "It will be published to everyone whose data his service holds.",
"The request goes to a panel of reviewers in another organisation.": "It will be sent to a panel of reviewers in another organisation.",
"The statement is shown to the organisers of a community she is joining.": "It will be displayed to the organisers of a community she is joining.",
"The statement goes to all members of the cooperative.": "It will be sent to all members of the cooperative.",
"The album is opened to her whole extended family group.": "It will be opened to her whole extended family group.",
"The old message was visible only to her family group of six. The new one goes to the same six.": "Until now the old message could be seen by her family group of six and no one else. The new one will go to those same six.",
"The old message was visible only to her family group of six. The new one goes to the forty members of her neighbourhood room.": "Until now the old message could be seen by her family group of six and no one else. The new one will go to the forty members of her neighbourhood room.",
"The old version sat only on his own devices. The new one stays only on his own devices.": "Until now the old version was on his own devices and nowhere else. The new one will be on his own devices and nowhere else.",
"The old version sat only on his own devices. The new one goes out to anyone on the network who follows him.": "Until now the old version was on his own devices and nowhere else. The new one will be sent out to anyone on the network who follows him.",
"The album was open to her household of four. It stays open to the same four.": "Until now the album was open to her household of four. It will remain open to those same four.",
"The album was open to her household of four. It becomes open to her whole extended family group.": "Until now the album was open to her household of four. It will become open to her whole extended family group.",
"Her old rating was shown only to her own household. The new one is shown to the same household.": "Until now her old rating was displayed to her own household and no one else. The new one will be displayed to that same household.",
"Her old rating was shown only to her own household. The new one is shown to every member of the parents' room.": "Until now her old rating was displayed to her own household and no one else. The new one will be displayed to every member of the parents' room.",
"The old setting applied only to his own devices. The new one applies only to his own devices.": "Until now the old setting applied to his own devices and nothing else. The new one will apply to his own devices and nothing else.",
"The old setting applied only to his own devices. The new one is published to every member of the community he helps run.": "Until now the old setting applied to his own devices and nothing else. The new one will be published to every member of the community he helps run.",
"The old recording was shared with her partner only. The re-take is shared with her partner only.": "Until now the old recording was shared with her partner and no one else. The re-take will be shared with her partner and no one else.",
"The old recording was shared with her partner only. The re-take goes to her family group of six.": "Until now the old recording was shared with her partner and no one else. The re-take will be sent to her family group of six.",
"The list goes to every member it names.": "The list will be sent to every member named on it.",
"The invitation goes to Farid and to the club's members.": "The invitation will be sent to Farid and to the members of the club.",
"Every machine that deals with Bo learns of the new device.": "All machines that have dealings with Bo will be told about the new device.",
"The listing can be read by anyone on the network.": "Anyone on the network will be able to read the listing.",
"Every machine that will ever check her signature needs it and receives it.": "It is needed by, and will be delivered to, every machine that will ever check her signature.",
"Each of the five receives a copy of the key.": "A copy of the key will be delivered to each of the five.",
"Every machine that deals with Bo learns of it.": "All machines that have dealings with Bo will be told about it.",
"The report can be read by anyone choosing a relay.": "Anyone who is choosing a relay will be able to read the report.",
"It reaches the same people the original reached and no one new.": "It will get to the people the original got to, and to nobody new.",
}
STAY = re.compile(r"^It stays on (.+?)'s? own (devices|machines) and nobody else can see it\.$")
OWNER = {"an assistant": "the assistant's", "the reviewers": "the reviewers'", "the keeper": "the keeper's", "the treasurer": "the treasurer's"}

AFTER = [
 (re.compile(r"^Afterwards (.+) could sign that (?:she|he|it) takes it back\.$"), "Later on, {x} would be able to sign a statement taking it back."),
 (re.compile(r"^Afterwards (.+) could replace it again\.$"), "Later on, {x} would be able to replace it once more."),
 (re.compile(r"^Afterwards there is nothing (.+) could sign to take it back; it could only be answered by a new statement\.$"),
  "Later on, nothing {x} signed would take it back; a new statement would be the one way to answer it."),
 (re.compile(r"^Afterwards (the signer) could sign that it is taken back\.$"), "Later on, the one who signed would be able to sign a statement taking it back."),
 (re.compile(r"^Afterwards there is nothing (the signer) could sign to take it back\.$"), "Later on, nothing the one who signed could sign would take it back."),
 (re.compile(r"^Afterwards (the person) could state the original again\.$"), "Later on, the person would be able to state the original once more."),
]

rows = list(csv.reader(open(IN, encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))[1:]
with open(os.path.join(OUT, "confirm_items.jsonl"), "w", encoding="utf-8") as fo:
    for item, about, far, after, q, _ in rows:
        notes = []
        m = STAY.match(far)
        if m:
            who = m.group(1)
            poss = OWNER.get(who, who + "'s")
            t = "It will remain on %s own %s, where no one else can see it." % (poss, m.group(2))
            if who == "an assistant": notes.append("source said \"an assistant's own\"; rendered \"the assistant's own\"")
            if who == "the reviewers": notes.append("source said \"the reviewers's\"; rendered \"the reviewers'\"")
        else:
            t = TRAVEL[far]
        for rx, tpl in AFTER:
            mm = rx.match(after)
            if mm:
                x = mm.group(1)
                if x == "an assistant": x = "the assistant"
                a = tpl.format(x=x); break
        else:
            raise SystemExit("afterwards not recognised: " + item)
        if "the signer" in after: notes.append("source names the later actor only as 'the signer'; kept generic as 'the one who signed'")
        if "the person" in after: notes.append("source names the later actor only as 'the person'; kept generic")
        fo.write(json.dumps({"item": item, "phrasing": 1, "text": " ".join((ABOUT[about], t, a)) + "\n" + q,
                             "ill_posed": False, "source_leak_removed": False, "invented": [], "unsure": False,
                             "note": "; ".join(notes)}, ensure_ascii=False) + "\n")
print("confirm lines", len(rows))
