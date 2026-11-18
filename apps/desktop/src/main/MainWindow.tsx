import { useEffect, useState } from "react";
import { getAppVersion } from "../lib/ipc";

export function MainWindow() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  return (
    <main className="placeholder">
      <h1>Ichnote</h1>
      <p>Leave a trace.</p>
      {version && <p className="version">v{version}</p>}
    </main>
  );
}
