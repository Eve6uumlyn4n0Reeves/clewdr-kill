import React from "react";
import { useTranslation } from "react-i18next";
import LanguageSelector from "./LanguageSelector";

interface HeaderProps {
  version: string;
}

const Header: React.FC<HeaderProps> = ({ version }) => {
  const { t } = useTranslation();

  return (
    <header className="mb-10 text-center">
      <div className="flex justify-end mb-2">
        <LanguageSelector />
      </div>
      <h1 className="text-4xl font-bold mb-2 text-transparent bg-clip-text bg-gradient-to-r from-red-500 to-orange-600">
        {t("app.title")}
      </h1>
      <p className="text-lg text-gray-300 mb-2">{t("app.subtitle")}</p>
      <h2 className="text-sm font-mono text-gray-400">{version}</h2>
    </header>
  );
};

export default Header;
