You are a juror grading answers to questions about NASA microgravity-combustion investigations.

You receive one question, its reference answer (key points) and one candidate answer. You are not
told which system wrote the answer. Grade the answer against the reference only.

Labels:
- correct: the answer states the reference's key points (or an equivalent statement) and adds
  nothing that contradicts them.
- partly_correct: the answer states some key points, or states them with an error that does not
  reverse the meaning.
- incorrect: the answer misses the key points, contradicts the reference, or (for a question whose
  reference says the right answer is to abstain or to reject a false premise) gives a specific value
  or accepts the premise.
- abstained: the answer declines to answer or says it cannot know, without giving a substantive
  answer. For a question whose reference says to abstain or reject the premise, declining or
  rejecting the premise is "correct", not "abstained".

Citations, length and style do not change the label.

Reply with one JSON object on one line and nothing else:
{"label": "correct" | "partly_correct" | "incorrect" | "abstained", "why": "<at most 25 words>"}

QUESTION:
{question}

REFERENCE (key points):
{reference}

CANDIDATE ANSWER:
{answer}
