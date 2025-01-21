#!/bin/bash

# Percorso della cartella con i file di input
input_dir="../../tests.nosync/cwida/"

# Nome del file di output (può essere modificato se necessario)
output_file="output.txt"

# File dei risultati
results_file="results3.txt"

# Pulizia del file dei risultati
echo "Risultati delle esecuzioni:" > "$results_file"

# Verifica che la cartella esista
if [[ ! -d "$input_dir" ]]; then
    echo "Errore: La cartella $input_dir non esiste."
    exit 1
fi

# Loop attraverso tutti i file nella cartella
for input_file in "$input_dir"*; do
    # Controlla se l'elemento è un file regolare
    if [[ -f "$input_file" ]]; then
        echo "Eseguendo ./fsst con input: $input_file"

        # Ottieni la dimensione del file di input in MB
        input_size_bytes=$(wc -c < "$input_file")
        input_size_mb=$(echo "scale=2; $input_size_bytes / 1024 / 1024" | bc)

        # Misura il tempo di esecuzione (usando gdate per compatibilità con macOS)
        start_time=$(gdate +%s.%N)
        ./fsst "$input_file"
        end_time=$(gdate +%s.%N)

        # Calcola il tempo trascorso
        elapsed_time=$(echo "$end_time - $start_time" | bc)

        # Calcola il rate in MB/s
        if (( $(echo "$elapsed_time > 0" | bc -l) )); then
            rate=$(echo "scale=2; $input_size_mb / $elapsed_time" | bc)
        else
            rate="Inf"  # Evita divisioni per zero
        fi

        # Ottieni la dimensione del file di output in MB
        if [[ -f "$output_file" ]]; then
            output_size_bytes=$(wc -c < "$output_file")
            output_size_mb=$(echo "scale=2; $output_size_bytes / 1024 / 1024" | bc)

            # Calcola il rapporto input/output
            if (( $(echo "$output_size_mb > 0" | bc -l) )); then
                ratio=$(echo "scale=2; $input_size_mb / $output_size_mb" | bc)
            else
                ratio="Inf"  # Evita divisioni per zero
            fi
        else
            output_size_mb="0.00"
            ratio="N/A"  # Il file di output non esiste
        fi

        # Salva il risultato nel file
        echo "$input_file: $input_size_mb MB, $elapsed_time secondi, $rate MB/s, rapporto input/output: $ratio" >> "$results_file"
    else
        echo "Elemento non processabile (non è un file): $input_file"
    fi
done

echo "Esecuzione completata. Risultati salvati in $results_file."