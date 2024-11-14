import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class analisisLexico {
    // Expresiones regulares
    private static final String VALID_LINE_REGEX = "^[ a-zA-Z0-9_\\-+*()\\[\\]#&/|=<>%:!]+;$";
    private static final String IDENTIFIER_REGEX = "[a-zA-Z_][a-zA-Z0-9_]{0,30}";
    private static final String RESERVED_WORDS_REGEX = "(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)";
    
    public static void main(String[] args) {
        // Ruta del archivo .txt
        String filePath = "prueba.txt"; // Cambiar por la ruta real del archivo

        try (BufferedReader br = new BufferedReader(new FileReader(filePath))) {
            StringBuilder contentBuilder = new StringBuilder();
            String line;

            while ((line = br.readLine()) != null) {
                contentBuilder.append(line).append(" "); // unir las líneas
            }

            String content = contentBuilder.toString();
            String[] logicalLines = content.split("(?<=;)");

            for (String logicalLine : logicalLines) {
                logicalLine = logicalLine.trim(); // quitar espacios

                if (!logicalLine.isEmpty()) {
                    if (isValidCLine(logicalLine)) {
                        System.out.println("La línea es válida en C: " + logicalLine);
                    } else {
                        System.out.println("La línea NO es válida en C: " + logicalLine);
                    }

                    String[] tokens = logicalLine.split("\\s+");
                    for (String token : tokens) {
                        if (isReservedWord(token)) {
                            System.out.println("Palabra reservada: " + token);
                        } else if (isIdentifier(token)) {
                            System.out.println("Identificador: " + token);
                        }
                    }
                }
            }
        } catch (IOException e) {
            e.getMessage();
            e.printStackTrace();
        }
    }

    public static boolean isValidCLine(String line) {
        Pattern pattern = Pattern.compile(VALID_LINE_REGEX);
        Matcher matcher = pattern.matcher(line);
        return matcher.matches();
    }

    public static boolean isIdentifier(String input) {
        return Pattern.matches(IDENTIFIER_REGEX, input);
    }

    public static boolean isReservedWord(String input) {
        return Pattern.matches(RESERVED_WORDS_REGEX, input);
    }
}
