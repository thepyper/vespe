# -*- coding: utf-8 -*-
"""
Questo script implementa una pipeline per generare un dataset di addestramento.
Utilizza un modello LLM "grande" (insegnante) per creare prompt e per etichettare
le risposte di un modello LLM "piccolo" (studente).

Il processo è il seguente:
1. Il modello GRANDE crea un prompt per l'utente che spinga il modello PICCOLO
   a usare un tool specifico.
2. Al modello PICCOLO viene fornito un system prompt normativo che descrive
   i tool disponibili e il formato di output atteso (es. JSON, XML, MCP).
3. Il modello PICCOLO risponde al prompt.
4. Il modello GRANDE analizza la conversazione (incluso il system prompt) e
   la etichetta, producendo un file JSON strutturato per l'addestramento.
"""

import argparse
import json
import os
import random
import requests

# --- DEFINIZIONE STRUTTURATA DEI TOOL ---
# Ogni tool è un dizionario con nome, descrizione e parametri.
# Questa struttura dati è la "source of truth" per generare le specifiche
# dei tool nei vari formati (JSON, XML, etc.).
TOOLS_DEFINITION = [
    {
        "name": "read_file",
        "description": "Legge e restituisce il contenuto di un file specificato.",
        "parameters": [
            {
                "name": "absolute_path",
                "type": "string",
                "description": "Il percorso assoluto del file da leggere."
            }
        ]
    },
    {
        "name": "write_file",
        "description": "Scrive del contenuto in un file specificato, sovrascrivendolo se esiste.",
        "parameters": [
            {
                "name": "file_path",
                "type": "string",
                "description": "Il percorso assoluto del file in cui scrivere."
            },
            {
                "name": "content",
                "type": "string",
                "description": "Il contenuto da scrivere nel file."
            }
        ]
    },
    {
        "name": "list_directory",
        "description": "Elenca i file e le cartelle in una data directory.",
        "parameters": [
            {
                "name": "path",
                "type": "string",
                "description": "Il percorso assoluto della directory da elencare."
            }
        ]
    },
    {
        "name": "run_shell_command",
        "description": "Esegue un comando nella shell di sistema.",
        "parameters": [
            {
                "name": "command",
                "type": "string",
                "description": "Il comando da eseguire."
            }
        ]
    }
]

# --- DEFINIZIONE DEI SYSTEM PROMPT ---

# Parte 1: Prompt Normativo (Fisso)
# Contiene le regole di comportamento generali per il modello.
NORMATIVE_SYSTEM_PROMPT = '''
Sei un assistente AI progettato per aiutare gli utenti eseguendo task.
Per farlo, hai a disposizione una serie di tool.

Il tuo processo di pensiero deve seguire questi passi:
1.  **THOUGHT**: Analizza la richiesta dell'utente e decidi se puoi rispondere direttamente o se hai bisogno di un tool. Se scegli un tool, spiega quale e perché.
2.  **TOOL_CODE**: Se hai deciso di usare un tool, scrivi la chiamata al tool nel formato specificato. **DEVI FERMARTI SUBITO DOPO AVER CHIAMATO IL TOOL.** Non devi generare la risposta del tool (TOOL_RESPONSE) o continuare la conversazione.

Regole Assolute:
-   Usa **solo e soltanto** i tool elencati nella sezione <tools>. Non inventare tool o parametri.
-   Rispetta **scrupolosamente** il formato di output richiesto per il TOOL_CODE.
-   Fermati **immediatamente** dopo aver prodotto il blocco TOOL_CODE.
'''

# Parte 2: Generatori di Specifiche Tool (Variabili)
# Queste funzioni creano la seconda parte del system prompt, che descrive i
# tool nel formato richiesto (MCP, JSON, XML).

def _build_mcp_tool_spec(tools):
    """Costruisce la specifica dei tool in formato MCP (Multi-Component Prompt)."""
    spec = "<tools>\n"
    for tool in tools:
        spec += f"  Tool: {tool['name']}\n"
        spec += f"    Description: {tool['description']}\n"
        spec += "    Parameters:\n"
        for param in tool['parameters']:
            spec += f"      - {param['name']} ({param['type']}): {param['description']}\n"
    spec += "</tools>\n\n"
    spec += "Formato di output per TOOL_CODE (MCP):\n"
    spec += "TOOL_CODE\n<tool_name>(\n  <param_name>='<value>',\n  ...\n)"
    return spec

def _build_json_tool_spec(tools):
    """Costruisce la specifica dei tool e il formato di output in JSON."""
    spec = "Definizione dei tool in formato JSON:\n"
    spec += json.dumps(tools, indent=2)
    spec += "\n\nFormato di output per TOOL_CODE (JSON):\n"
    spec += "```json\n"
    spec += "{\n"
    spec += '  "tool_name": "<nome del tool>",\n'
    spec += '  "parameters": {\n'
    spec += '    "<nome parametro>": "<valore>",\n'
    spec += '    ...\n'
    spec += '  }\n'
    spec += "}
"
    spec += "```\n"
    return spec

def _build_xml_tool_spec(tools):
    """Costruisce la specifica dei tool e il formato di output in XML."""
    spec = "<tools>\n"
    for tool in tools:
        spec += f'  <tool name="{tool[