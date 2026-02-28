export type CustomToolForm = {
  name: string;
  id: string;
  skillsDir: string;
  rulesFile: string;
  icon: string;
};

export const EMPTY_CUSTOM_TOOL_FORM: CustomToolForm = {
  name: "",
  id: "",
  skillsDir: "",
  rulesFile: "",
  icon: "",
};
