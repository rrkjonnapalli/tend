export type PersonRow = {
  id: string;
  firstName: string;
  lastName: string;
  nickname: string | null;
  dobYear: number | null;
  dobMonth: number | null;
  dobDay: number | null;
  phone: string | null;
  email: string | null;
};

export type ContactKind = 'phone' | 'email' | 'address';
export type Contact = { id: string; kind: ContactKind; value: string; label: string };
export type Person = PersonRow & { tob: string | null; notes: string; contacts: Contact[] };

/** The name fields a display name is built from. */
export type PersonName = Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>;

export type PersonPatch = Partial<{
  firstName: string;
  lastName: string;
  nickname: string | null;
  dob: { year: number | null; month: number; day: number } | null;
  tob: string | null;
  notes: string;
}>;

export type ContactFields = { value: string; label: string };
