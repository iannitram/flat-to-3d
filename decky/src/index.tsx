import { definePlugin } from "@decky/api";
import { Field, staticClasses } from "@decky/ui";
import { FaVrCardboard } from "react-icons/fa";
import { useEffect, useState } from "react";
import { view } from "./engine";
import type { LinkView } from "./types";

function Content() {
  const [value, setValue] = useState<LinkView | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;
    const poll = async () => {
      const response = await view();
      if (!mounted) return;
      if (response.ok) {
        setValue(response.data);
        setError(null);
      } else {
        setValue(null);
        setError(response.error.message);
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), 1000);
    return () => {
      mounted = false;
      window.clearInterval(timer);
    };
  }, []);

  if (error !== null) return <Field label="Flat to 3D" description={error} />;
  if (value === null) return <Field label="Flat to 3D" description="Loading…" />;
  return <Field label={value.label} description={value.hint} />;
}

export default definePlugin(() => ({
  name: "Flat to 3D",
  titleView: <div className={staticClasses.Title}>Flat to 3D</div>,
  content: <Content />,
  icon: <FaVrCardboard />,
}));
