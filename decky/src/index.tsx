import { definePlugin } from "@decky/api";
import { staticClasses } from "@decky/ui";
import { FaVrCardboard } from "react-icons/fa";
import { Content } from "./panel";
import { startToasts } from "./toasts";
import { syncCollection } from "./collection";
import { syncLaunchOptions } from "./launch";

export default definePlugin(() => {
  const stop = startToasts();
  void syncCollection().then((plan) =>
    "code" in plan ? [plan] : syncLaunchOptions(plan),
  );
  const titleView = <div className={staticClasses.Title}>Flat to 3D</div>;

  return {
    name: "Flat to 3D",
    titleView,
    content: <Content />,
    icon: <FaVrCardboard />,
    onDismount: stop,
  };
});
