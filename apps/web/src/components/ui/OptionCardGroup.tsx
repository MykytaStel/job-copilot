type OptionCardGroupItem<T extends string> = {
  id: T;
  label: string;
};

export function OptionCardGroup<T extends string>({
  options,
  value,
  onToggle,
}: {
  options: Array<OptionCardGroupItem<T>>;
  value: T[];
  onToggle: (value: T) => void;
}) {
  return (
    <div className="optionGrid">
      {options.map((option) => (
        <label key={option.id} className="optionCard">
          <input
            type="checkbox"
            checked={value.includes(option.id)}
            onChange={() => onToggle(option.id)}
          />
          <span>{option.label}</span>
        </label>
      ))}
    </div>
  );
}
