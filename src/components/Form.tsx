import { Button, FormControlLabel, Switch, TextField } from '@mui/material';
import BigNumber from 'bignumber.js';
import { RefObject, useRef } from 'react';
import { Unit } from '../bindings';

export interface FormProps {
  fields: FormField[];
  submitName: string;
  values: Record<string, FormValue>;
  setValues: (values: Record<string, FormValue>) => void;
  onSubmit: () => void;
}

export type FormField = FormFieldProps &
  (FormFieldText | FormFieldAmount | FormFieldOption);
export type FormValue = string | boolean;

export interface FormFieldProps {
  id: string;
  label: string;
}

export interface FormFieldText {
  type: 'text';
}

export interface FormFieldAmount {
  type: 'amount';
  unit: Unit;
}

export interface FormFieldOption {
  type: 'option';
}

export default function Form({
  fields,
  submitName,
  values,
  setValues,
  onSubmit,
}: FormProps) {
  const refs: Record<string, RefObject<HTMLInputElement>> = {};
  const fieldIds = fields
    .map((field) => field.id)
    .sort((a, b) => a.localeCompare(b));

  for (const fieldId of fieldIds) {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    refs[fieldId] = useRef<HTMLInputElement>(null);
  }

  let allValid = true;

  return (
    <>
      {fields.map((field, i) => {
        const first = i === 0;
        const inputRef = refs[field.id];

        const nextField = fields[i + 1];
        const nextRef = nextField ? refs[nextField.id] : undefined;

        const focusNext = (canSubmit: boolean) => {
          if (nextRef !== undefined) {
            nextRef.current?.focus();
          } else if (canSubmit) {
            // submit
          }
        };

        switch (field.type) {
          case 'text':
          case 'amount': {
            const value = (values[field.id] as string) ?? '';

            let valid = value.length > 0;

            if (valid && field.type === 'amount') {
              const amount = BigNumber(value);

              valid =
                !amount.isNaN() &&
                amount.isGreaterThanOrEqualTo(0) &&
                amount.isFinite();

              const amountMojos = amount.multipliedBy(
                BigNumber(10).pow(field.unit.decimals),
              );
              valid &&=
                amountMojos.decimalPlaces() === 0 &&
                amountMojos.isLessThanOrEqualTo(
                  BigNumber('18446744073709551615'),
                );
            }

            if (!valid) {
              allValid = false;
            }

            return (
              <TextField
                key={field.id}
                sx={{ mt: first ? 0 : 2 }}
                inputRef={inputRef}
                label={field.label}
                autoFocus={first}
                fullWidth
                value={value}
                error={value.length > 0 && !valid}
                onChange={(event) =>
                  setValues({ ...values, [field.id]: event.target.value })
                }
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    focusNext(true);
                  }
                }}
              />
            );
          }

          case 'option': {
            return (
              <FormControlLabel
                key={field.id}
                sx={{ mt: first ? 0 : 2 }}
                autoFocus={first}
                control={
                  <Switch
                    inputRef={inputRef}
                    checked={(values[field.id] as boolean) ?? false}
                    onChange={(event) =>
                      setValues({ ...values, [field.id]: event.target.checked })
                    }
                  />
                }
                label={field.label}
              />
            );
          }
        }
      })}

      <Button
        sx={{ mt: fields.length > 0 ? 2 : 0 }}
        variant='contained'
        fullWidth
        disabled={!allValid}
        onClick={() => {
          if (allValid) {
            onSubmit();
          }
        }}
      >
        {submitName}
      </Button>
    </>
  );
}
