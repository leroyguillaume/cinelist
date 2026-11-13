import { type FormEvent, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { BackIcon, CloseIcon, PlusIcon } from "../components/Icons";
import { eliminationsEach, filmsToAdd, MAX_FILMS, MAX_PARTICIPANTS } from "../lib/game";
import { useCreateList } from "../lib/queries";

export function parseTitles(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

type ParticipantRow = {
  id: number;
  name: string;
};

export function NewListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const create = useCreateList();
  const [name, setName] = useState("");
  const nextParticipantId = useRef(2);
  const [participants, setParticipants] = useState<ParticipantRow[]>([
    { id: 0, name: "" },
    { id: 1, name: "" },
  ]);
  const [films, setFilms] = useState("");

  const names = participants.map((participant) => participant.name.trim()).filter(Boolean);
  const titles = parseTitles(films);
  const missing = filmsToAdd(titles.length, participants.length);

  const updateParticipant = (id: number, value: string) =>
    setParticipants((current) =>
      current.map((participant) =>
        participant.id === id ? { ...participant, name: value } : participant,
      ),
    );

  const addParticipant = () => {
    const id = nextParticipantId.current++;
    setParticipants((current) => [...current, { id, name: "" }]);
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    create.mutate(
      { name: name.trim(), participants: names, films: titles.slice(0, MAX_FILMS) },
      { onSuccess: (list) => navigate(`/lists/${list.id}`, { replace: true }) },
    );
  };

  return (
    <main className="page" style={{ maxWidth: 640 }}>
      <div className="page-header">
        <Link className="icon-button page-back" to="/" aria-label={t("app.back")}>
          <BackIcon />
        </Link>
        <h1>{t("newList.title")}</h1>
      </div>
      <form className="stack" style={{ gap: 22 }} onSubmit={submit}>
        <div className="field">
          <label htmlFor="list-name">{t("newList.name")}</label>
          <input
            id="list-name"
            className="input"
            required
            maxLength={100}
            placeholder={t("newList.namePlaceholder")}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        </div>

        <fieldset className="field" style={{ border: "none", padding: 0, margin: 0 }}>
          <legend className="field-label">{t("newList.participants")}</legend>
          <p className="hint">{t("newList.participantsHint")}</p>
          {participants.map((participant, index) => (
            <div className="row" key={participant.id}>
              <span className="chip-number">{index + 1}</span>
              <input
                className="input"
                aria-label={`${t("newList.participantPlaceholder")} ${index + 1}`}
                placeholder={t("newList.participantPlaceholder")}
                maxLength={40}
                required={index === 0}
                value={participant.name}
                onChange={(event) => updateParticipant(participant.id, event.target.value)}
              />
              {participants.length > 1 && (
                <button
                  type="button"
                  className="icon-button"
                  aria-label={t("newList.removeParticipant", {
                    name: participant.name || index + 1,
                  })}
                  onClick={() =>
                    setParticipants((current) =>
                      current.filter((other) => other.id !== participant.id),
                    )
                  }
                >
                  <CloseIcon size={18} />
                </button>
              )}
            </div>
          ))}
          {participants.length < MAX_PARTICIPANTS && (
            <button
              type="button"
              className="button button-dashed button-small"
              onClick={addParticipant}
            >
              <PlusIcon size={16} />
              {t("newList.addParticipant")}
            </button>
          )}
        </fieldset>

        <div className="field">
          <label htmlFor="list-films">{t("newList.films")}</label>
          <textarea
            id="list-films"
            className="input"
            required
            value={films}
            onChange={(event) => setFilms(event.target.value)}
          />
          <p className="hint">{t("newList.filmsHint")}</p>
          {titles.length > 0 && (
            <p className="hint" aria-live="polite">
              {missing > 0
                ? t("newList.missing", { count: missing })
                : t("newList.turns", {
                    films: titles.length,
                    count: eliminationsEach(titles.length, participants.length),
                  })}
            </p>
          )}
        </div>

        <ErrorText error={create.error} />
        <button type="submit" className="button button-primary" disabled={create.isPending}>
          {t("newList.submit")}
        </button>
      </form>
    </main>
  );
}
