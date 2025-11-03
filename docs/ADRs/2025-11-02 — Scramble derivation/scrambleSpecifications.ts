export type EventID = string;

export const scrambleSpecifications: Record<
  EventID,
  | {
      // `true` (default): this event has its own scramble generator.
      // string value: this event uses the scramble generator with this ID.
      monoscramble?: true | EventID;
      canonicalAlgGenerationInfo?: any; // TODO
      randomPatternGenerationInfo?: any; // TODO
      filteringInfo?: any; // TODO
    }
  | {
      monoscramble: false;
      subevents: Array<{
        eventID: EventID;
        fixedNumberOfScrambles?: number;
      }>;
    }
> = {
  "333": {},
  "333oh": {
    monoscramble: "333",
  },
  "333bf": {},
  "333mbf": {
    monoscramble: false,
    subevents: [
      {
        eventID: "333bf",
      },
    ],
  },
  "unofficial-guildford": {
    monoscramble: false,
    subevents: [
      { eventID: "222", fixedNumberOfScrambles: 1 },
      { eventID: "333", fixedNumberOfScrambles: 1 },
      // …
      { eventID: "333oh", fixedNumberOfScrambles: 1 },
      // …
      { eventID: "sq1", fixedNumberOfScrambles: 1 },
    ],
  },
};
