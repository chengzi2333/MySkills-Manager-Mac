import { useEffect, useState } from "react";

import {
  onboardingComplete,
  onboardingSetSkillsDir,
  setupStatus,
  type OnboardingCompleteResult,
  type ToolStatus,
} from "../api/tauri";
import "./OnboardingWizard.css";

type Props = {
  initialSkillsDir: string;
  initialAutoSync: boolean;
  onCompleted: (result: OnboardingCompleteResult) => void;
};

export default function OnboardingWizard({
  initialSkillsDir,
  initialAutoSync,
  onCompleted,
}: Props) {
  const [step, setStep] = useState(1);
  const [skillsDir, setSkillsDir] = useState(initialSkillsDir);
  const [autoSync, setAutoSync] = useState(initialAutoSync);
  const [skillsCount, setSkillsCount] = useState(0);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void setupStatus()
      .then(setTools)
      .catch(() => setTools([]));
  }, []);

  async function handleCheckSkillsDir() {
    setBusy(true);
    setStatus("checking skills directory...");
    try {
      const result = await onboardingSetSkillsDir(skillsDir);
      setSkillsCount(result.skills.length);
      setStep(2);
      setStatus("");
    } catch (e: unknown) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleFinish() {
    setBusy(true);
    setStatus("saving onboarding state...");
    try {
      const result = await onboardingComplete(autoSync);
      setStatus("");
      onCompleted(result);
    } catch (e: unknown) {
      setStatus(String(e));
      setBusy(false);
    }
  }

  const availableTools = tools.filter((item) => item.exists).length;

  return (
    <div className="onboarding-screen">
      <div className="onboarding-card animate-fadein">
        <h1 className="onboarding-title">Welcome to MySkills Manager</h1>
        <p className="onboarding-subtitle">
          Step {step} / 3 · Configure your workspace before entering the app.
        </p>

        {step === 1 && (
          <section className="onboarding-step">
            <h2>1. Choose Skills Directory</h2>
            <p>Set the directory that contains your skill folders and SKILL.md files.</p>
            <input
              className="field-input"
              value={skillsDir}
              onChange={(e) => setSkillsDir(e.target.value)}
              placeholder="C:\\Users\\Keith\\my-skills"
            />
            <div className="onboarding-actions">
              <button
                className="btn btn-primary"
                onClick={() => void handleCheckSkillsDir()}
                disabled={busy || skillsDir.trim().length === 0}
              >
                {busy ? "Checking..." : "Next"}
              </button>
            </div>
          </section>
        )}

        {step === 2 && (
          <section className="onboarding-step">
            <h2>2. Select Sync Mode</h2>
            <p>
              Found <strong>{skillsCount}</strong> skills and <strong>{availableTools}</strong> available tools.
            </p>
            <label className="onboarding-toggle">
              <input
                type="checkbox"
                checked={autoSync}
                onChange={(e) => setAutoSync(e.target.checked)}
              />
              <span>Enable auto-sync now (otherwise manual sync later in Tools page)</span>
            </label>
            <div className="onboarding-actions">
              <button className="btn btn-ghost" onClick={() => setStep(1)} disabled={busy}>
                Back
              </button>
              <button className="btn btn-primary" onClick={() => setStep(3)} disabled={busy}>
                Next
              </button>
            </div>
          </section>
        )}

        {step === 3 && (
          <section className="onboarding-step">
            <h2>3. Confirm</h2>
            <ul className="onboarding-list">
              <li>Skills Directory: {skillsDir}</li>
              <li>Detected Skills: {skillsCount}</li>
              <li>Detected Tools: {availableTools}</li>
              <li>Sync Mode: {autoSync ? "Auto-sync on completion" : "Manual sync later"}</li>
            </ul>
            <div className="onboarding-actions">
              <button className="btn btn-ghost" onClick={() => setStep(2)} disabled={busy}>
                Back
              </button>
              <button className="btn btn-primary" onClick={() => void handleFinish()} disabled={busy}>
                {busy ? "Finishing..." : "Finish"}
              </button>
            </div>
          </section>
        )}

        {status && <p className="onboarding-status">{status}</p>}
      </div>
    </div>
  );
}
