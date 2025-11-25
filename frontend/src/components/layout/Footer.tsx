import React from "react";
import { useTranslation } from "react-i18next";

const Footer: React.FC = () => {
  const { t } = useTranslation();
  const currentYear = new Date().getFullYear();

  return (
    <footer className="mt-12 text-center text-gray-500 text-sm">
      <p>{t("app.footer", { year: currentYear })}</p>
    </footer>
  );
};

export default Footer;
