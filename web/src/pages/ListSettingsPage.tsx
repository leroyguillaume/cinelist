import { type FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { BackIcon, CloseIcon } from "../components/Icons";
import { Loading } from "../components/Loading";
import { MAX_PARTICIPANTS } from "../lib/game";
import {
  useAddParticipant,
  useDeleteList,
  useList,
  useRemoveParticipant,
  useRenameParticipant,
  useUpdateList,
} from "../lib/queries";
import type { List, Participant } from "../lib/schemas";

function GeneralForm({ list }: { list: List }) {
  const { t } = useTranslation();
  const update = useUpdateList(list.id);
  const [name, setName] = useState(list.name);
  useEffect(() => {
    setName(list.name);
  }, [list.name]);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    update.mutate({ name: name.trim() });
  };
  return (
    <form className="card stack" onSubmit={submit}>
      <h2>{t("settings.general")}</h2>
      <div className="field">
        <label htmlFor="settings-name">{t("settings.name")}</label>
        <input
          id="settings-name"
          className="input"
          required
          maxLength={100}
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
      </div>
      <ErrorText error={update.error} />
      {update.isSuccess && <p className="hint">{t("settings.saved")}</p>}
      <button type="submit" className="button button-primary" disabled={update.isPending}>
        {t("app.save")}
      </button>
    </form>
  );
}

function ParticipantRow({ list, participant }: { list: List; participant: Participant }) {
  const { t } = useTranslation();
  const rename = useRenameParticipant(list.id);
  const remove = useRemoveParticipant(list.id);
  const [name, setName] = useState(participant.name);
  const idle = list.activeSelection === null;
  const submit = (event: FormEvent) => {
    event.preventDefault();
    rename.mutate({ id: participant.id, name: name.trim() });
  };
  return (
    <li className="stack" style={{ gap: 6 }}>
      <form className="inline-form" onSubmit={submit}>
        <input
          className="input"
          aria-label={t("settings.participantName")}
          required
          maxLength={40}
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        {name.trim() !== participant.name && (
          <button type="submit" className="button button-primary button-small">
            {t("app.save")}
          </button>
        )}
        {idle && list.participants.length > 1 && (
          <button
            type="button"
            className="icon-button"
            aria-label={t("settings.removeParticipant", { name: participant.name })}
            onClick={() => {
              if (
                window.confirm(t("settings.confirmRemoveParticipant", { name: participant.name }))
              ) {
                remove.mutate(participant.id);
              }
            }}
          >
            <CloseIcon size={18} />
          </button>
        )}
      </form>
      <ErrorText error={rename.error ?? remove.error} />
    </li>
  );
}

function ParticipantsForm({ list }: { list: List }) {
  const { t } = useTranslation();
  const add = useAddParticipant(list.id);
  const [name, setName] = useState("");
  const idle = list.activeSelection === null;
  const submit = (event: FormEvent) => {
    event.preventDefault();
    add.mutate(name.trim(), { onSuccess: () => setName("") });
  };
  return (
    <section className="card stack">
      <h2>{t("settings.participants")}</h2>
      <ol className="stack" style={{ listStyle: "none", margin: 0, padding: 0, gap: 10 }}>
        {list.participants.map((participant) => (
          <ParticipantRow key={participant.id} list={list} participant={participant} />
        ))}
      </ol>
      {idle && list.participants.length < MAX_PARTICIPANTS && (
        <form className="stack" style={{ gap: 6 }} onSubmit={submit}>
          <div className="inline-form">
            <input
              className="input"
              aria-label={t("settings.participantName")}
              placeholder={t("settings.participantName")}
              required
              maxLength={40}
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
            <button type="submit" className="button button-primary" disabled={add.isPending}>
              {t("settings.addParticipant")}
            </button>
          </div>
          <ErrorText error={add.error} />
        </form>
      )}
    </section>
  );
}

export function ListSettingsPage() {
  const { t } = useTranslation();
  const { listId = "" } = useParams();
  const navigate = useNavigate();
  const list = useList(listId);
  const remove = useDeleteList(listId);
  if (list.isPending) {
    return (
      <main className="page">
        <Loading />
      </main>
    );
  }
  if (list.isError) {
    return (
      <main className="page">
        <ErrorText error={list.error} />
      </main>
    );
  }
  return (
    <main className="page" style={{ maxWidth: 720 }}>
      <div className="page-header">
        <Link className="icon-button" to={`/lists/${listId}`} aria-label={t("app.back")}>
          <BackIcon />
        </Link>
        <h1>{t("settings.title")}</h1>
      </div>
      {list.data.activeSelection && <p className="notice">{t("list.frozen")}</p>}
      <GeneralForm list={list.data} />
      <ParticipantsForm list={list.data} />
      <section className="card stack">
        <h2>{t("settings.danger")}</h2>
        <button
          type="button"
          className="button button-danger"
          disabled={remove.isPending}
          onClick={() => {
            if (window.confirm(t("settings.confirmDeleteList", { name: list.data.name }))) {
              remove.mutate(undefined, { onSuccess: () => navigate("/", { replace: true }) });
            }
          }}
        >
          {t("settings.deleteList")}
        </button>
        <ErrorText error={remove.error} />
      </section>
    </main>
  );
}
